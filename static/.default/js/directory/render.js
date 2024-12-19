import { formatFileSize } from '../utils/format.js';
import { getFileIcon } from './icons.js';

export function renderDirectoryContent(content) {
    const gridElement = document.querySelector('.directory-grid');
    if (!gridElement) return;

    gridElement.innerHTML = '';

    // Add parent directory link if not in root
    if (content.path !== '/') {
        gridElement.appendChild(createDirectoryItem({
            name: '..',
            type: 'directory',
            path: getParentPath(content.path)
        }));
    }

    // Sort items: directories first, then files
    const sortedItems = [...content.items].sort((a, b) => {
        if (a.type === b.type) return a.name.localeCompare(b.name);
        return a.type === 'directory' ? -1 : 1;
    });

    sortedItems.forEach(item => {
        gridElement.appendChild(createDirectoryItem(item));
    });
}

function createDirectoryItem(item) {
    const element = document.createElement('div');
    element.className = 'directory-item';
    element.dataset.path = item.path;
    element.dataset.type = item.type;

    const icon = document.createElement('div');
    icon.className = 'item-icon';
    icon.textContent = getFileIcon(item.type, item.name);

    const name = document.createElement('div');
    name.className = 'item-name';
    name.textContent = item.name;

    element.appendChild(icon);
    element.appendChild(name);

    if (item.type === 'file' && item.size !== undefined) {
        const size = document.createElement('div');
        size.className = 'item-size';
        size.textContent = formatFileSize(item.size);
        element.appendChild(size);
    }

    return element;
}

function getParentPath(path) {
    return path.split('/').slice(0, -1).join('/') || '/';
}