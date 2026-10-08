<script setup lang="ts">
import { UAParser } from 'ua-parser-js'
import { useData } from 'vitepress'
import { VPButton } from 'vitepress/theme'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

const REPO = 'https://github.com/creeperkatze/full-steam-ahead'
const FLATHUB = 'https://flathub.org/apps/dev.creeperkatze.full-steam-ahead'

interface NavigatorUaData {
	getHighEntropyValues(hints: string[]): Promise<{ architecture?: string }>
}

const { theme } = useData()
const version = computed(() => theme.value.version as string | undefined)

type Os = 'windows' | 'macos' | 'linux' | null
type Arch = 'x64' | 'arm64'

const OS_LABELS: Record<Exclude<Os, null>, string> = {
	windows: 'Windows',
	macos: 'macOS',
	linux: 'Linux',
}

const os = ref<Os>(null)
const arch = ref<Arch>('x64')
const open = ref(false)
const openOther = ref(false)
const root = ref<HTMLElement | null>(null)
const otherRoot = ref<HTMLElement | null>(null)

function detectOs(): Os {
	const { os: osResult, device } = new UAParser(navigator.userAgent).getResult()
	if (device.type === 'mobile' || device.type === 'tablet') return null
	if (osResult.name === 'Windows') return 'windows'
	if (osResult.name === 'macOS') return 'macos'
	if (!osResult.name || osResult.name === 'iOS' || osResult.name === 'Android') return null
	return 'linux'
}

function detectArch(): Arch {
	const { cpu } = new UAParser(navigator.userAgent).getResult()
	return cpu.architecture === 'arm' || cpu.architecture === 'arm64' ? 'arm64' : 'x64'
}

function onClickOutside(event: MouseEvent) {
	if (open.value && root.value && !root.value.contains(event.target as Node)) open.value = false
	if (openOther.value && otherRoot.value && !otherRoot.value.contains(event.target as Node)) {
		openOther.value = false
	}
}

onMounted(async () => {
	os.value = detectOs()
	arch.value = detectArch()

	const uaData = (navigator as unknown as { userAgentData?: NavigatorUaData }).userAgentData
	if (uaData?.getHighEntropyValues) {
		try {
			const { architecture } = await uaData.getHighEntropyValues(['architecture'])
			if (architecture === 'arm') arch.value = 'arm64'
		} catch {
			// Client hints unavailable/denied, keep the ua-parser-js result.
		}
	}

	document.addEventListener('click', onClickOutside)
})

onBeforeUnmount(() => document.removeEventListener('click', onClickOutside))

function asset(name: string) {
	return `${REPO}/releases/download/v${version.value}/${name}`
}

const primary = computed(() => {
	if (!version.value || !os.value) return { label: 'Download', url: `${REPO}/releases/latest` }

	const v = version.value
	const suffix = os.value === 'windows' && arch.value === 'arm64' ? ' (ARM64)' : ''
	const label = `Download for ${OS_LABELS[os.value]}${suffix}`

	if (os.value === 'windows') {
		return { label, url: asset(`full-steam-ahead-${v}-windows-${arch.value}-setup.exe`) }
	}
	if (os.value === 'macos') {
		return { label, url: asset(`full-steam-ahead-${v}-darwin-universal.dmg`) }
	}
	return { label, url: FLATHUB }
})

interface Method {
	label: string
	url: string
}

interface MethodGroup {
	os: Exclude<Os, null>
	label: string
	methods: Method[]
}

function methodsFor(targetOs: Exclude<Os, null>, v: string): Method[] {
	if (targetOs === 'windows') {
		return [
			{ label: 'Installer (x64)', url: asset(`full-steam-ahead-${v}-windows-x64-setup.exe`) },
			{
				label: 'Installer (ARM64)',
				url: asset(`full-steam-ahead-${v}-windows-arm64-setup.exe`),
			},
			{ label: 'MSI (x64)', url: asset(`full-steam-ahead-${v}-windows-x64.msi`) },
			{ label: 'MSI (ARM64)', url: asset(`full-steam-ahead-${v}-windows-arm64.msi`) },
			{ label: 'Portable (x64)', url: asset(`full-steam-ahead-${v}-windows-x64.exe`) },
			{ label: 'Portable (ARM64)', url: asset(`full-steam-ahead-${v}-windows-arm64.exe`) },
		]
	}
	if (targetOs === 'macos') {
		return [{ label: 'Universal .dmg', url: asset(`full-steam-ahead-${v}-darwin-universal.dmg`) }]
	}
	return [
		{ label: 'Flathub', url: FLATHUB },
		{ label: 'AppImage (x64)', url: asset(`full-steam-ahead-${v}-linux-amd64.AppImage`) },
		{
			label: 'AppImage (ARM64)',
			url: asset(`full-steam-ahead-${v}-linux-aarch64.AppImage`),
		},
		{ label: '.deb (x64)', url: asset(`full-steam-ahead-${v}-linux-amd64.deb`) },
		{ label: '.deb (ARM64)', url: asset(`full-steam-ahead-${v}-linux-arm64.deb`) },
		{ label: '.rpm (x64)', url: asset(`full-steam-ahead-${v}-linux-x86_64.rpm`) },
		{ label: '.rpm (ARM64)', url: asset(`full-steam-ahead-${v}-linux-aarch64.rpm`) },
	]
}

const methods = computed<Method[]>(() => {
	if (!version.value || !os.value) return []
	return methodsFor(os.value, version.value)
})

const otherGroups = computed<MethodGroup[]>(() => {
	if (!version.value) return []
	const v = version.value

	const platforms: Exclude<Os, null>[] = ['windows', 'macos', 'linux']
	const others = os.value ? platforms.filter((p) => p !== os.value) : platforms

	return others.map((p) => ({ os: p, label: OS_LABELS[p], methods: methodsFor(p, v) }))
})
</script>

<template>
	<div class="hero-actions">
		<div class="hero-action">
			<div ref="root" class="install-group">
				<VPButton
					tag="a"
					size="medium"
					theme="brand"
					:text="primary.label"
					:href="primary.url"
					:class="['install-button', os && `os-${os}`, methods.length && 'has-toggle']"
				/>
				<button
					v-if="methods.length"
					type="button"
					class="install-toggle"
					:aria-expanded="open"
					aria-label="Other download methods"
					@click="open = !open"
				>
					<svg viewBox="0 0 24 24" width="14" height="14">
						<path
							d="M6 9l6 6 6-6"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
					</svg>
				</button>
				<div v-if="open" class="install-menu">
					<a
						v-for="option in methods"
						:key="option.url"
						:href="option.url"
						class="install-menu-item"
						@click="open = false"
					>
						{{ option.label }}
					</a>
				</div>
			</div>
		</div>
		<div v-if="otherGroups.length" class="hero-action">
			<div ref="otherRoot" class="other-platforms-group">
				<button
					type="button"
					class="other-platforms-button"
					:aria-expanded="openOther"
					@click="openOther = !openOther"
				>
					Other platforms
					<svg viewBox="0 0 24 24" width="14" height="14">
						<path
							d="M6 9l6 6 6-6"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
					</svg>
				</button>
				<div v-if="openOther" class="install-menu">
					<template v-for="group in otherGroups" :key="group.os">
						<div class="install-menu-heading">{{ group.label }}</div>
						<a
							v-for="option in group.methods"
							:key="option.url"
							:href="option.url"
							class="install-menu-item"
							@click="openOther = false"
						>
							{{ option.label }}
						</a>
					</template>
				</div>
			</div>
		</div>
		<div class="hero-action">
			<VPButton
				tag="a"
				size="medium"
				theme="alt"
				text="GitHub"
				:href="REPO"
				class="github-button"
			/>
		</div>
	</div>
</template>

<style scoped>
.hero-actions {
	display: flex;
	flex-wrap: wrap;
	margin: -6px;
	padding-top: 24px;
}

.hero-action {
	flex-shrink: 0;
	padding: 6px;
}

@media (min-width: 640px) {
	.hero-actions {
		padding-top: 32px;
	}
}

.install-group {
	position: relative;
	display: inline-flex;
	align-items: center;
}

a.install-button {
	position: relative;
}

a.github-button {
	display: inline-flex;
	align-items: center;
	gap: 8px;
}

a.github-button::before {
	content: '';
	width: 16px;
	height: 16px;
	flex-shrink: 0;
	background-color: currentColor;
	-webkit-mask: url('./icons/github.svg') no-repeat center / contain;
	mask: url('./icons/github.svg') no-repeat center / contain;
}

a.install-button.os-windows,
a.install-button.os-macos,
a.install-button.os-linux {
	padding-left: 44px !important;
}

a.install-button.os-windows::before,
a.install-button.os-macos::before,
a.install-button.os-linux::before {
	content: '';
	position: absolute;
	left: 18px;
	top: 50%;
	width: 16px;
	height: 16px;
	transform: translateY(-50%);
	background-color: currentColor;
}

a.install-button.os-windows::before {
	-webkit-mask: url('./icons/windows.svg') no-repeat center / contain;
	mask: url('./icons/windows.svg') no-repeat center / contain;
}

a.install-button.os-macos::before {
	-webkit-mask: url('./icons/apple.svg') no-repeat center / contain;
	mask: url('./icons/apple.svg') no-repeat center / contain;
}

a.install-button.os-linux::before {
	-webkit-mask: url('./icons/linux.svg') no-repeat center / contain;
	mask: url('./icons/linux.svg') no-repeat center / contain;
}

a.install-button.has-toggle {
	padding-right: 14px !important;
	border-top-right-radius: 0 !important;
	border-bottom-right-radius: 0 !important;
}

.install-toggle {
	display: inline-flex;
	height: 40px;
	width: 32px;
	flex-shrink: 0;
	align-items: center;
	justify-content: center;
	margin-left: -1px;
	border: 1px solid var(--vp-button-brand-border);
	border-left-color: rgba(255, 255, 255, 0.3);
	border-top-right-radius: 20px;
	border-bottom-right-radius: 20px;
	background-color: var(--vp-button-brand-bg);
	color: var(--vp-button-brand-text);
	transition:
		border-color 0.25s,
		background-color 0.25s;
}

.install-toggle:hover {
	border-color: var(--vp-button-brand-hover-border);
	background-color: var(--vp-button-brand-hover-bg);
	color: var(--vp-button-brand-hover-text);
}

.other-platforms-group {
	position: relative;
	display: inline-flex;
	align-items: center;
}

.other-platforms-button {
	display: inline-flex;
	height: 40px;
	align-items: center;
	gap: 6px;
	padding: 0 18px;
	border: 1px solid var(--vp-c-divider);
	border-radius: 20px;
	background-color: transparent;
	color: var(--vp-c-text-1);
	font-size: 14px;
	font-weight: 600;
	transition:
		border-color 0.25s,
		color 0.25s;
}

.other-platforms-button:hover {
	border-color: var(--vp-c-brand-1);
	color: var(--vp-c-brand-1);
}

.install-menu {
	position: absolute;
	top: calc(100% + 8px);
	left: 0;
	z-index: 10;
	display: flex;
	min-width: 200px;
	max-height: 70vh;
	flex-direction: column;
	gap: 2px;
	padding: 8px;
	overflow-y: auto;
	border: 1px solid var(--vp-c-divider);
	border-radius: 12px;
	background-color: var(--vp-c-bg-elv);
	box-shadow: var(--vp-shadow-3);
	text-align: left;
}

.install-menu-heading {
	padding: 8px 10px 2px;
	font-size: 11px;
	font-weight: 600;
	color: var(--vp-c-text-3);
	text-transform: uppercase;
	letter-spacing: 0.04em;
}

.install-menu-heading:first-child {
	padding-top: 2px;
}

.install-menu-item {
	padding: 6px 10px;
	border-radius: 6px;
	font-size: 14px;
	color: var(--vp-c-text-1) !important;
	text-decoration: none !important;
}

.install-menu-item:hover {
	background-color: var(--vp-c-bg-soft);
	color: var(--vp-c-brand-1) !important;
}
</style>
