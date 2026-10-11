import { getCurrentWindow } from '@tauri-apps/api/window'

import { commands } from '../bindings'

// Rounds the page like the GTK frame, which is square while maximized, fullscreen or tiled.
export async function roundWindowCorners() {
	const root = document.documentElement
	// Set before the first paint, since a later layout change can leave stale pixels behind
	root.classList.add('rounded-window')
	const update = async () => {
		root.classList.toggle('rounded-window', await commands.windowCornersRounded())
	}
	await update()
	await getCurrentWindow().onResized(update)
}
