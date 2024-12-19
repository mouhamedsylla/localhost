import { formatFileSize } from '../utils/format.js';

export function updateStats(content) {
    const statsElement = document.querySelector('.directory-stats');
    if (!statsElement) return;

    const stats = calculateStats(content.items);
    
    statsElement.innerHTML = `
        Files: ${stats.files} | 
        Directories: ${stats.directories} | 
        Total Size: ${formatFileSize(stats.totalSize)}
    `;
}

function calculateStats(items) {
    return items.reduce((acc, item) => ({
        files: acc.files + (item.type === 'file' ? 1 : 0),
        directories: acc.directories + (item.type === 'directory' ? 1 : 0),
        totalSize: acc.totalSize + (item.size || 0)
    }), { files: 0, directories: 0, totalSize: 0 });
}