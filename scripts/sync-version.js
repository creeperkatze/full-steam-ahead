import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const rootDir = join(dirname(fileURLToPath(import.meta.url)), '..')
const cargoPath = join(rootDir, 'src-tauri/Cargo.toml')

const { version } = JSON.parse(readFileSync(join(rootDir, 'package.json'), 'utf8'))

execFileSync('cargo', ['set-version', version, '--manifest-path', cargoPath], {
	stdio: 'inherit',
})

execFileSync('git', ['add', 'src-tauri/Cargo.toml', 'src-tauri/Cargo.lock'], { cwd: rootDir })
