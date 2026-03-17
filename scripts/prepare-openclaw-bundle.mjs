#!/usr/bin/env node

import fs from "node:fs";
import fsp from "node:fs/promises";
import path from "node:path";
import { spawnSync, spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const desktopRoot = path.resolve(__dirname, "..");
const workspaceRoot = path.resolve(desktopRoot, "..");
const openclawRoot = path.resolve(workspaceRoot, "openclaw");
const bundleDir = path.resolve(desktopRoot, "src-tauri", "bundle", "resources", "openclaw-bundle");
const tempDir = path.resolve(desktopRoot, ".tmp", "openclaw-bundle");
const OPENCLAW_MIN_NODE = "24.14.0";//"22.16.0";
const RUN_MAX_BUFFER = Number(process.env.OPENCLAW_BUNDLE_RUN_MAX_BUFFER || 128 * 1024 * 1024);
const REQUESTED_NODE_ARCH = normalizeNodeArch(process.env.OPENCLAW_BUNDLE_NODE_ARCH);

function normalizeNodeArch(rawArch) {
    const value = String(rawArch ?? "").trim().toLowerCase();
    if (!value) {
        return null;
    }
    if (value === "x64" || value === "x86_64" || value === "amd64") {
        return "x64";
    }
    if (value === "arm64" || value === "aarch64") {
        return "arm64";
    }
    if (value === "ia32" || value === "x86") {
        return "ia32";
    }
    throw new Error(`Unsupported OPENCLAW_BUNDLE_NODE_ARCH value: ${rawArch}`);
}

function run(cmd, args, opts = {}) {
    console.log(`[bundle] Executing: ${cmd} ${args.join(" ")}`);
    const useShell = process.platform === "win32" && cmd === "npm";
    const result = spawnSync(cmd, args, {
        cwd: opts.cwd,
        env: opts.env ?? process.env,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
        shell: useShell,
        windowsHide: true,
        maxBuffer: RUN_MAX_BUFFER
    });

    console.log(`[bundle] Command exited with status: ${result.status}`);
    
    if (result.error) {
        console.error(`[bundle] Command error: ${String(result.error)}`);
        throw new Error(`${cmd} ${args.join(" ")} failed\n${String(result.error)}`);
    }

    if (result.status !== 0) {
        const out = (result.stdout ?? "").trim();
        const err = (result.stderr ?? "").trim();
        console.error(`[bundle] Command stdout: ${out}`);
        console.error(`[bundle] Command stderr: ${err}`);
        const detail = [out, err].filter(Boolean).join("\n");
        throw new Error(`${cmd} ${args.join(" ")} failed${detail ? `\n${detail}` : ""}`);
    }

    const output = (result.stdout ?? "").trim();
    //console.log(`[bundle] Command output: ${output}`);
    return output;
}

async function runAsync(cmd, args, opts = {}) {
    console.log(`[bundle] Executing (async): ${cmd} ${args.join(" ")}`);
    const useShell = process.platform === "win32" && cmd === "npm";
    
    return new Promise((resolve, reject) => {
        const child = spawn(cmd, args, {
            cwd: opts.cwd,
            env: opts.env ?? process.env,
            stdio: ["ignore", "pipe", "pipe"],
            shell: useShell,
            windowsHide: true
        });

        let stdout = "";
        let stderr = "";

        child.stdout.on("data", (data) => {
            const chunk = data.toString();
            stdout += chunk;
            process.stdout.write(chunk);
        });

        child.stderr.on("data", (data) => {
            const chunk = data.toString();
            stderr += chunk;
            process.stderr.write(chunk);
        });

        child.on("close", (code) => {
            console.log(`[bundle] Command exited with status: ${code}`);
            if (code !== 0) {
                reject(new Error(`${cmd} ${args.join(" ")} failed with exit code ${code}\n${stderr}`));
            } else {
                resolve(stdout.trim());
            }
        });

        child.on("error", (error) => {
            console.error(`[bundle] Command error: ${String(error)}`);
            reject(new Error(`${cmd} ${args.join(" ")} failed\n${String(error)}`));
        });
    });
}

function parseVersion(v) {
    const match = String(v).trim().match(/^v?(\d+)\.(\d+)\.(\d+)/);
    if (!match) {
        return null;
    }
    return {
        major: Number(match[1]),
        minor: Number(match[2]),
        patch: Number(match[3])
    };
}

function versionGte(left, right) {
    const a = parseVersion(left);
    const b = parseVersion(right);
    if (!a || !b) {
        return false;
    }
    if (a.major !== b.major) return a.major > b.major;
    if (a.minor !== b.minor) return a.minor > b.minor;
    return a.patch >= b.patch;
}

function ensureFile(p, label) {
    if (!fs.existsSync(p) || !fs.statSync(p).isFile()) {
        throw new Error(`${label} not found: ${p}`);
    }
}

function resolveNpmDir() {
    const npmRoot = run("npm", ["root", "-g"]);
    const candidate = path.join(npmRoot, "npm");
    if (fs.existsSync(candidate)) {
        return candidate;
    }

    const prefix = run("npm", ["config", "get", "prefix"]);
    const extra = process.platform === "win32"
        ? path.join(prefix, "node_modules", "npm")
        : path.join(prefix, "lib", "node_modules", "npm");
    if (fs.existsSync(extra)) {
        return extra;
    }

    throw new Error("Unable to locate npm directory for offline bundle");
}

function resolveInstalledOpenclaw(prefix) {
    const candidates = process.platform === "win32"
        ? [
            path.join(prefix, "bin", "openclaw.cmd"),
            path.join(prefix, "bin", "openclaw.exe"),
            path.join(prefix, "node_modules", "openclaw", "openclaw.mjs"),
            path.join(prefix, "lib", "node_modules", "openclaw", "openclaw.mjs"),
            path.join(prefix, "node_modules", ".bin", "openclaw.cmd")
        ]
        : [
            path.join(prefix, "bin", "openclaw"),
            path.join(prefix, "node_modules", "openclaw", "openclaw.mjs"),
            path.join(prefix, "lib", "node_modules", "openclaw", "openclaw.mjs"),
            path.join(prefix, "node_modules", ".bin", "openclaw")
        ];

    for (const candidate of candidates) {
        try {
            if (fs.statSync(candidate).isFile()) {
                return candidate;
            }
        } catch { }
    }

    throw new Error(`bundled openclaw executable not found under: ${prefix}`);
}

async function ensureCleanDir(p) {
    await fsp.rm(p, { recursive: true, force: true });
    await fsp.mkdir(p, { recursive: true });
}

async function ensureUserWritableRecursive(rootDir) {
    const queue = [rootDir];
    while (queue.length > 0) {
        const current = queue.pop();
        const stat = await fsp.lstat(current);
        await fsp.chmod(current, stat.mode | 0o200);
        if (!stat.isDirectory()) {
            continue;
        }
        const children = await fsp.readdir(current);
        for (const child of children) {
            queue.push(path.join(current, child));
        }
    }
}

function npmPack(args, cwd) {
    const raw = run("npm", ["pack", "--json", ...args], { cwd });
    let parsed;
    try {
        parsed = JSON.parse(raw);
    } catch (error) {
        throw new Error(`Failed to parse npm pack --json output: ${String(error)}\n${raw}`);
    }

    const record = Array.isArray(parsed) ? parsed[0] : parsed;
    const filename = record?.filename;
    if (!filename || typeof filename !== "string") {
        throw new Error(`npm pack --json missing filename: ${raw}`);
    }

    const packedFile = path.resolve(cwd, filename);
    ensureFile(packedFile, "packed openclaw tarball");
    return {
        filename,
        packedFile,
        version: typeof record?.version === "string" ? record.version : null
    };
}

function localOpenclawBuildReady() {
    const packageJsonPath = path.join(openclawRoot, "package.json");
    if (!fs.existsSync(packageJsonPath)) {
        return false;
    }
    const hasDistEntryJs = fs.existsSync(path.join(openclawRoot, "dist", "entry.js"));
    const hasDistEntryMjs = fs.existsSync(path.join(openclawRoot, "dist", "entry.mjs"));
    return hasDistEntryJs || hasDistEntryMjs;
}

function resolveLocalOpenclawVersion() {
    const packageJsonPath = path.join(openclawRoot, "package.json");
    if (!fs.existsSync(packageJsonPath)) {
        return null;
    }
    try {
        const pkg = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
        return typeof pkg?.version === "string" ? pkg.version : null;
    } catch {
        return null;
    }
}

function packOpenclawTarball() {
    const localVersion = resolveLocalOpenclawVersion();

    if (localOpenclawBuildReady()) {
        console.log("[bundle] packing openclaw from local source (dist ready)...");
        const localPack = npmPack(["--ignore-scripts"], openclawRoot);
        return {
            ...localPack,
            source: "local-source",
            cleanup: async () => fsp.rm(localPack.packedFile, { force: true })
        };
    }

    const spec = localVersion ? `openclaw@${localVersion}` : "openclaw@latest";
    console.log(`[bundle] local source missing dist, fetching ${spec} from npm registry...`);
    const remotePack = npmPack([spec], tempDir);
    return {
        ...remotePack,
        source: `npm-registry:${spec}`,
        cleanup: async () => { }
    };
}

async function resolveBundledNodeRuntime() {
    const customNode = process.env.OPENCLAW_BUNDLE_NODE;
    if (customNode && fs.existsSync(customNode)) {
        const customVersion = run(customNode, ["-v"]);
        let customArch = process.arch;
        try {
            customArch = run(customNode, ["-p", "process.arch"]).trim() || process.arch;
        } catch {
            customArch = process.arch;
        }
        if (versionGte(customVersion, OPENCLAW_MIN_NODE)) {
            return {
                nodePath: customNode,
                nodeVersion: customVersion,
                nodeSource: "env:OPENCLAW_BUNDLE_NODE",
                nodeArch: customArch
            };
        }
    }

    console.log(`[bundle] provisioning portable node@${OPENCLAW_MIN_NODE} runtime...`);
    if (REQUESTED_NODE_ARCH) {
        console.log(`[bundle] requested node arch: ${REQUESTED_NODE_ARCH}`);
    }
    const nodeProvisionPrefix = path.join(tempDir, "node-runtime");
    await ensureCleanDir(nodeProvisionPrefix);
    const installEnv = { ...process.env };
    if (REQUESTED_NODE_ARCH) {
        installEnv.npm_config_arch = REQUESTED_NODE_ARCH;
    }
    run("npm", [
        "install",
        "--prefix",
        nodeProvisionPrefix,
        "--no-audit",
        "--no-fund",
        "--loglevel=error",
        `node@${OPENCLAW_MIN_NODE}`
    ], { env: installEnv });

    const bundledNodePath = process.platform === "win32"
        ? path.join(nodeProvisionPrefix, "node_modules", "node", "bin", "node.exe")
        : path.join(nodeProvisionPrefix, "node_modules", "node", "bin", "node");
    ensureFile(bundledNodePath, "bundled node runtime");

    const effectiveNodeArch = REQUESTED_NODE_ARCH ?? process.arch;
    const isCrossArchBinary = effectiveNodeArch !== process.arch;
    let bundledNodeVersion;
    if (isCrossArchBinary) {
        console.log(
            `[bundle] skip executing cross-arch node binary (${effectiveNodeArch}) on host ${process.arch}`
        );
        bundledNodeVersion = `v${OPENCLAW_MIN_NODE}`;
    } else {
        bundledNodeVersion = run(bundledNodePath, ["-v"]);
        if (!versionGte(bundledNodeVersion, OPENCLAW_MIN_NODE)) {
            throw new Error(
                `bundled node ${bundledNodeVersion} does not satisfy >=${OPENCLAW_MIN_NODE}`
            );
        }
    }

    return {
        nodePath: bundledNodePath,
        nodeVersion: bundledNodeVersion,
        nodeSource: `npm:node@${OPENCLAW_MIN_NODE}`,
        nodeArch: effectiveNodeArch
    };
}

async function resolveBundledNpmDir() {
    const npmProvisionPrefix = path.join(tempDir, "npm-runtime");
    await ensureCleanDir(npmProvisionPrefix);
    run("npm", [
        "install",
        "--prefix",
        npmProvisionPrefix,
        "--no-audit",
        "--no-fund",
        "--loglevel=error",
        "npm@10"
    ]);

    const npmDir = path.join(npmProvisionPrefix, "node_modules", "npm");
    const npmCli = path.join(npmDir, "bin", "npm-cli.js");
    ensureFile(npmCli, "bundled npm cli");
    return npmDir;
}

async function main() {
    console.log(`[bundle] Starting at: ${new Date().toISOString()}`);
    const start = Date.now();
    
    if (process.env.OPENCLAW_DESKTOP_SKIP_BUNDLE_PREP === "1") {
        console.log("[bundle] skip prepare because OPENCLAW_DESKTOP_SKIP_BUNDLE_PREP=1");
        return;
    }

    await ensureCleanDir(bundleDir);
    await ensureCleanDir(tempDir);

    console.log(`[bundle] Cleanup done in ${(Date.now() - start) / 1000}s`);
    
    const packed = packOpenclawTarball();
    
    console.log(`[bundle] packOpenclawTarball done in ${(Date.now() - start) / 1000}s`);

    const runtime = await resolveBundledNodeRuntime();
    
    console.log(`[bundle] resolveBundledNodeRuntime done in ${(Date.now() - start) / 1000}s`);
    
    const bundledTgz = path.join(bundleDir, "openclaw.tgz");
    await fsp.copyFile(packed.packedFile, bundledTgz);
    await packed.cleanup();

    console.log(`[bundle] Openclaw tgz copied in ${(Date.now() - start) / 1000}s`);

    console.log("[bundle] copying node runtime and npm...");
    const nodeDir = path.join(bundleDir, "node");
    await ensureCleanDir(nodeDir);
    const nodeTarget = path.join(nodeDir, process.platform === "win32" ? "node.exe" : "node");
    await fsp.copyFile(runtime.nodePath, nodeTarget);
    if (process.platform !== "win32") {
        await fsp.chmod(nodeTarget, 0o755);
    }
    ensureFile(nodeTarget, "bundled node runtime");

    console.log(`[bundle] Node runtime copied in ${(Date.now() - start) / 1000}s`);

    let npmDir;
    try {
        npmDir = resolveNpmDir();
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        console.log(`[bundle] system npm dir unavailable (${message}), provisioning standalone npm...`);
        npmDir = await resolveBundledNpmDir();
    }
    const npmTarget = path.join(bundleDir, "npm");
    await fsp.rm(npmTarget, { recursive: true, force: true });
    await fsp.cp(npmDir, npmTarget, {
        recursive: true,
        // npm package ships a root .npmrc; tauri resource scanner may fail on it on some hosts.
        filter: (src) => path.basename(src) !== ".npmrc"
    });

    console.log(`[bundle] NPM copied in ${(Date.now() - start) / 1000}s`);

    console.log("[bundle] warming offline npm cache...");
    const cacheDir = path.join(bundleDir, "npm-cache");
    const installPrefix = path.join(tempDir, "install-prefix");
    await fsp.mkdir(cacheDir, { recursive: true });
    console.log("[bundle] Cache directory created:", cacheDir);
    let prefixAvailable = false;
    const bundledPrefix = path.join(bundleDir, "prefix");
    await fsp.rm(bundledPrefix, { recursive: true, force: true });
    console.log("[bundle] Bundled prefix cleaned");

    console.log(`[bundle] Cache setup done in ${(Date.now() - start) / 1000}s`);

    try {
        console.log("[bundle] Running npm install...");
        console.log("[bundle] Install prefix:", installPrefix);
        console.log("[bundle] Bundle tgz:", bundledTgz);
        const npmInstallStart = Date.now();
        await runAsync("npm", [
            "install",
            "--prefix",
            installPrefix,
            bundledTgz,
            "--cache",
            cacheDir,
            "--no-audit",
            "--no-fund",
            "--loglevel=error"
        ]);
        console.log(`[bundle] npm install completed in ${(Date.now() - npmInstallStart) / 1000}s`);

        console.log("[bundle] snapshot installed prefix for fully-offline install...");
        const snapshotStart = Date.now();
        // npm local install 会产生指向临时目录的绝对软链；这里解引用，避免打包后出现失效链接。
        await fsp.cp(installPrefix, bundledPrefix, { recursive: true, dereference: true });
        console.log(`[bundle] Prefix snapshot copied in ${(Date.now() - snapshotStart) / 1000}s`);

        if (process.env.OPENCLAW_BUNDLE_SKIP_VERIFY === "1") {
            console.log("[bundle] skip prefix verification because OPENCLAW_BUNDLE_SKIP_VERIFY=1");
        } else if (runtime.nodeArch !== process.arch) {
            console.log(
                `[bundle] skip prefix verification for cross-arch node bundle ${runtime.nodeArch} on host ${process.arch}`
            );
        } else {
            console.log("[bundle] verifying bundled prefix snapshot...");
            const verifyStart = Date.now();
            const verifyPrefix = path.join(tempDir, "verify-prefix");
            await fsp.cp(bundledPrefix, verifyPrefix, { recursive: true });
            const verifyOpenclaw = resolveInstalledOpenclaw(verifyPrefix);
            const verifyEnv = {
                ...process.env,
                PATH: `${path.dirname(nodeTarget)}${path.delimiter}${process.env.PATH || ""}`
            };
            if (process.platform === "win32") {
                run("cmd", ["/C", verifyOpenclaw, "--version"], { env: verifyEnv });
            } else {
                run(verifyOpenclaw, ["--version"], { env: verifyEnv });
            }
            await fsp.rm(verifyPrefix, { recursive: true, force: true });
            console.log(`[bundle] Prefix verification done in ${(Date.now() - verifyStart) / 1000}s`);
        }
        prefixAvailable = true;
        console.log("[bundle] Prefix available set to true");
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        console.warn(`[bundle] WARN: prefix snapshot failed, fallback to npm-cache mode: ${message}`);
        await fsp.rm(bundledPrefix, { recursive: true, force: true });
        // 仍然确保 tgz 被写入离线 cache；运行时可通过 npm-cli + cache + tgz 完成离线安装。
        run("npm", ["cache", "add", bundledTgz, "--cache", cacheDir, "--loglevel=error"]);
    } finally {
        await fsp.rm(installPrefix, { recursive: true, force: true });
    }

    console.log(`[bundle] NPM install and prefix setup done in ${(Date.now() - start) / 1000}s`);

    // npm tarballs may preserve read-only bits. Ensure resources stay writable so
    // repeated local builds can overwrite copied files without EACCES.
    await ensureUserWritableRecursive(bundleDir);

    console.log(`[bundle] Permissions fixed in ${(Date.now() - start) / 1000}s`);

    console.log("[bundle] reorganizing npm into node directory...");
    const npmSourceDir = path.join(bundleDir, "npm");
    const npmBinDir = path.join(npmSourceDir, "bin");
    const npmBinFiles = await fsp.readdir(npmBinDir, { withFileTypes: true });
    for (const dirent of npmBinFiles) {
        if (!dirent.isFile()) {
            continue;
        }
        const srcPath = path.join(npmBinDir, dirent.name);
        const destPath = path.join(nodeDir, dirent.name);
        await fsp.copyFile(srcPath, destPath);
        if (process.platform !== "win32") {
            await fsp.chmod(destPath, 0o755);
        }
    }
    const nodeModulesDir = path.join(nodeDir, "node_modules");
    await fsp.mkdir(nodeModulesDir, { recursive: true });
    const npmTargetDir = path.join(nodeModulesDir, "npm");
    await fsp.rename(npmSourceDir, npmTargetDir);

    console.log(`[bundle] NPM reorganized in ${(Date.now() - start) / 1000}s`);

    const npmCli = path.join(nodeDir, "node_modules", "npm", "bin", "npm-cli.js");
    const manifest = {
        name: "openclaw-offline-bundle",
        generatedAt: new Date().toISOString(),
        openclawVersion: packed.version ?? resolveLocalOpenclawVersion() ?? "unknown",
        openclawSource: packed.source,
        nodeVersion: runtime.nodeVersion,
        nodeSource: runtime.nodeSource,
        nodePlatform: `${process.platform}-${runtime.nodeArch}`,
        prefixAvailable,
        files: {
            openclawTgz: "openclaw.tgz",
            npmCache: "npm-cache",
            node: path.relative(bundleDir, nodeTarget),
            npmCli: path.relative(bundleDir, npmCli),
            prefix: prefixAvailable ? "prefix" : null
        }
    };
    await fsp.writeFile(
        path.join(bundleDir, "manifest.json"),
        JSON.stringify(manifest, null, 2),
        "utf8"
    );

    console.log(`[bundle] Manifest generated in ${(Date.now() - start) / 1000}s`);

    await fsp.rm(tempDir, { recursive: true, force: true });
    console.log("[bundle] ready:", bundleDir);
    console.log(`[bundle] All done in ${(Date.now() - start) / 1000}s`);
}

main().catch((error) => {
    console.error("[bundle] failed:", error instanceof Error ? error.message : String(error));
    process.exit(1);
});