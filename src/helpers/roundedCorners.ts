import { getCurrentWindow } from '@tauri-apps/api/window'

import { commands } from '../bindings'

// Rounds the page like the GTK frame, which is square while maximized, fullscreen or tiled.
export async function roundWindowCorners() {
	const update = async () => {
		const rounded = await commands.windowCornersRounded()
		document.documentElement.classList.toggle('rounded-window', rounded)
	}
	await update()
	await getCurrentWindow().onResized(update)
}
