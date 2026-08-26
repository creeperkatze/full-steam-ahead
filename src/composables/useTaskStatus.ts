import { ref } from 'vue'

const loading = ref(false)
const status = ref('Ready')

async function runTask<T>(label: string, task: () => Promise<T>): Promise<T | undefined> {
	loading.value = true
	status.value = label
	try {
		return await task()
	} catch {
		return undefined
	} finally {
		loading.value = false
		status.value = 'Ready'
	}
}

export function useTaskStatus() {
	return {
		loading,
		status,
		runTask,
	}
}
