import { getCurrentWindow } from '@tauri-apps/api/window'

// Rounds the window corners, except while maximized.
export async function roundWindowCorners() {
	const win = getCurrentWindow()
	const update = async () => {
		document.documentElement.classList.toggle('rounded-window', !(await win.isMaximized()))
	}
	await update()
	await win.onResized(update)
}
