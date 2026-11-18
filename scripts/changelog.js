import { execFileSync } from 'node:child_process'
import { readFileSync, writeFileSync } from 'node:fs'

// Usage: node scripts/changelog.js check | release | notes <version>
const [command, version] = process.argv.slice(2)
const changelog = readFileSync('CHANGELOG.md', 'utf8').replace(/\r\n/g, '\n')

const fail = (message) => {
	console.error(`error: ${message}`)
	process.exit(1)
}

// A section ends at the next heading or at the compare links at the bottom
const notesFor = (name) => {
	const section = changelog.split(/^## /m).find((part) => part.startsWith(`[${name}]`)) ?? ''
	return section
		.slice(section.indexOf('\n') + 1)
		.split(/^\[.+\]: /m)[0]
		.trim()
}

if (command === 'check') {
	const notes = notesFor('Unreleased')
	if (!notes) fail('Nothing under ## [Unreleased] in CHANGELOG.md')
	console.log(notes)
} else if (command === 'release') {
	// Runs from the version hook, after pnpm bumped package.json
	const next = JSON.parse(readFileSync('package.json', 'utf8')).version
	const tag = `v${next}`
	const date = new Date().toISOString().slice(0, 10)
	writeFileSync(
		'CHANGELOG.md',
		changelog
			.replace('## [Unreleased]', `## [Unreleased]\n\n## [${next}] - ${date}`)
			.replace(
				/^\[Unreleased\]: (.+)\/compare\/(.+)\.\.\.HEAD$/m,
				`[Unreleased]: $1/compare/${tag}...HEAD\n[${next}]: $1/compare/$2...${tag}`,
			),
	)
	execFileSync('git', ['add', 'CHANGELOG.md'])
} else if (command === 'notes') {
	const notes = notesFor(version?.replace(/^v/, ''))
	if (!notes) fail(`No notes for ${version} in CHANGELOG.md`)
	console.log(notes)
} else {
	fail('Usage: node scripts/changelog.js check | release | notes <version>')
}
