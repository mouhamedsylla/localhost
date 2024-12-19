const FILE_ICONS = {
    directory: '📁',
    default: '📄',
    image: '🖼️',
    video: '🎥',
    audio: '🎵',
    pdf: '📕',
    archive: '📦',
    code: '📝'
};

const FILE_EXTENSIONS = {
    image: ['.jpg', '.jpeg', '.png', '.gif', '.svg', '.webp'],
    video: ['.mp4', '.webm', '.avi', '.mov'],
    audio: ['.mp3', '.wav', '.ogg'],
    pdf: ['.pdf'],
    archive: ['.zip', '.rar', '.7z', '.tar', '.gz'],
    code: ['.js', '.css', '.html', '.tsx', '.jsx', '.py', '.java', '.cpp']
};

export function getFileIcon(type, filename = '') {
    if (type === 'directory') return FILE_ICONS.directory;
    
    const extension = filename.toLowerCase().split('.').pop();
    for (const [category, extensions] of Object.entries(FILE_EXTENSIONS)) {
        if (extensions.includes(`.${extension}`)) {
            return FILE_ICONS[category];
        }
    }
    
    return FILE_ICONS.default;
}