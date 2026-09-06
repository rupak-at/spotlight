import { readFileSync } from 'node:fs';
import { argv, stdout } from 'node:process';

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function readCargoPackageVersion(path) {
  const contents = readFileSync(path, 'utf8');
  const packageStart = contents.indexOf('[package]');
  const packageRemainder = contents.slice(packageStart + '[package]'.length);
  const nextSection = packageRemainder.search(/^\[/m);
  const packageSection =
    nextSection === -1 ? packageRemainder : packageRemainder.slice(0, nextSection);
  const version = packageSection?.match(/^version\s*=\s*"([^"]+)"\s*$/m)?.[1];

  if (packageStart === -1 || !version) {
    throw new Error(`Could not read the [package] version from ${path}`);
  }

  return version;
}

const packageJson = readJson('package.json');
const packageLock = readJson('package-lock.json');
const tauriConfig = readJson('src-tauri/tauri.conf.json');
const versions = new Map([
  ['package.json', packageJson.version],
  ['package-lock.json', packageLock.version],
  ['package-lock.json root package', packageLock.packages?.['']?.version],
  ['src-tauri/Cargo.toml', readCargoPackageVersion('src-tauri/Cargo.toml')],
  ['src-tauri/tauri.conf.json', tauriConfig.version],
]);

const expectedVersion = packageJson.version;
const mismatches = [...versions].filter(([, version]) => version !== expectedVersion);

if (mismatches.length > 0) {
  const details = [...versions]
    .map(([file, version]) => `  ${file}: ${version ?? 'missing'}`)
    .join('\n');
  throw new Error(`Release versions are not aligned:\n${details}`);
}

if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(expectedVersion)) {
  throw new Error(`Invalid release version: ${expectedVersion}`);
}

const releaseTag = argv[2];
if (releaseTag && releaseTag !== `v${expectedVersion}`) {
  throw new Error(
    `Release tag ${releaseTag} does not match application version v${expectedVersion}`,
  );
}

stdout.write(
  releaseTag
    ? `Release metadata matches ${releaseTag}.\n`
    : `Release metadata is aligned at v${expectedVersion}.\n`,
);
