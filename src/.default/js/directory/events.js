export function setupEventListeners() {
    document.querySelector('.directory-grid')?.addEventListener('click', handleItemClick);
}

function handleItemClick(event) {
    const item = event.target.closest('.directory-item');
    if (!item) return;

    const path = item.dataset.path;
    const type = item.dataset.type;

    if (type === 'directory') {
        window.location.href = `${path}`;
    } else if (type === 'file') {
        window.location.href = path;
    }
}