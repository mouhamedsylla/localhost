const FILE_ICONS = {
    directory: '📁',
    default: '📄',
    image: '🖼️',
    video: '🎥',
    audio: '🎵',
    pdf: '📕',
    archive: '📦'
};

const CODE_ICONS = {
    // Web
    '.html': '🌐',
    '.css': '🎨',
    '.js': '📜',
    '.jsx': '⚛️',
    '.tsx': '⚛️',
    '.ts': '💠',
    
    // Scripting
    '.py': '🐍',
    '.rb': '💎',
    '.php': '🐘',
    
    // Compiled
    '.java': '☕',
    '.cpp': '⚙️',
    '.c': '🔧',
    '.cs': '🎯',
    '.rs': '🦀',
    '.go': '🐹',
    
    // Data/Config
    '.json': '📊',
    '.yml': '⚙️',
    '.xml': '📑',
    '.md': '📝',
    
    // Shell
    '.sh': '🐚',
    '.bash': '🐚'
};

const FILE_EXTENSIONS = {
    image: ['.jpg', '.jpeg', '.png', '.gif', '.svg', '.webp'],
    video: ['.mp4', '.webm', '.avi', '.mov'],
    audio: ['.mp3', '.wav', '.ogg'],
    pdf: ['.pdf'],
    archive: ['.zip', '.rar', '.7z', '.tar', '.gz']
};

export function getFileIcon(type, filename = '') {
    if (type === 'directory') return FILE_ICONS.directory;
    
    const extension = filename.toLowerCase().split('.').pop();
    const fullExtension = `.${extension}`;
    
    // Check for specific programming language icon
    if (CODE_ICONS[fullExtension]) {
        return CODE_ICONS[fullExtension];
    }
    
    // Check other file categories
    for (const [category, extensions] of Object.entries(FILE_EXTENSIONS)) {
        if (extensions.includes(fullExtension)) {
            return FILE_ICONS[category];
        }
    }
    
    return FILE_ICONS.default;
}