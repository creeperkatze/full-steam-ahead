import { ref } from 'vue'

const loading = ref(false)
const error = ref('')
const status = ref('Ready')

async function runTask<T>(label: string, task: () => Promise<T>): Promise<T | undefined> {
	loading.value = true
	error.value = ''
	status.value = label
	try {
		return await task()
	} catch (err) {
		error.value = commandMessage(err)
	} finally {
		loading.value = false
		status.value = 'Ready'
	}
}

function commandMessage(err: unknown) {
	if (typeof err === 'object' && err && 'message' in err) {
		return String((err as { message: unknown }).message)
	}
	return String(err)
}

export function useTaskStatus() {
	return {
		loading,
		error,
		status,
		runTask,
	}
}
