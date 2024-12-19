export async function loadDirectoryContent(path) {
    try {
        const response = await fetch(`/api/directory?path=${encodeURIComponent(path)}`);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error('Error loading directory content:', error);
        throw error;
    }
}