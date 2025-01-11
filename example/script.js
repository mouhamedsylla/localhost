document.addEventListener('DOMContentLoaded', () => {
    const dropZone = document.getElementById('dropZone');
    const fileInput = document.getElementById('fileInput');
    const filesList = document.getElementById('filesList');
    const pagination = document.getElementById('pagination');
    const prevBtn = document.getElementById('prevBtn');
    const nextBtn = document.getElementById('nextBtn');
    const currentPageSpan = document.getElementById('currentPage');
    const totalPagesSpan = document.getElementById('totalPages');
    const searchInput = document.getElementById('searchInput');
    const sortSelect = document.getElementById('sortSelect');
    
    // Variables globales
    let allFiles = [];
    let filteredFiles = [];
    const ITEMS_PER_PAGE = 4;
    let currentPage = 1;
    
    // Créer le conteneur de notifications
    const notificationsContainer = document.createElement('div');
    notificationsContainer.className = 'notifications-container';
    document.body.appendChild(notificationsContainer);

    // Gestion de la recherche et du tri
    searchInput.addEventListener('input', () => {
        currentPage = 1; // Reset page on search
        refreshFilesList();
    });

    sortSelect.addEventListener('change', refreshFilesList);

    // Amélioration de filterAndSortFiles
    function filterAndSortFiles() {
        const searchTerm = searchInput.value.toLowerCase();
        filteredFiles = allFiles.filter(file => 
            file && file.name && file.name.toLowerCase().includes(searchTerm)
        );

        const sortMethod = sortSelect.value;
        filteredFiles.sort((a, b) => {
            if (!a || !b) return 0;
            
            switch(sortMethod) {
                case 'name-asc':
                    return a.name.localeCompare(b.name);
                case 'name-desc':
                    return b.name.localeCompare(a.name);
                case 'size-asc':
                    return (a.size || 0) - (b.size || 0);
                case 'size-desc':
                    return (b.size || 0) - (a.size || 0);
                case 'date-asc':
                    return new Date(a.uploadDate || 0) - new Date(b.uploadDate || 0);
                case 'date-desc':
                    return new Date(b.uploadDate || 0) - new Date(a.uploadDate || 0);
                default:
                    return 0;
            }
        });
    }

    // Gestion de la pagination
    function updatePagination() {
        const totalPages = Math.ceil(filteredFiles.length / ITEMS_PER_PAGE);
        
        // Afficher/masquer la pagination selon le nombre de fichiers
        pagination.style.display = filteredFiles.length > ITEMS_PER_PAGE ? 'flex' : 'none';
        
        if (pagination.style.display === 'flex') {
            currentPageSpan.textContent = currentPage;
            totalPagesSpan.textContent = totalPages;
            prevBtn.disabled = currentPage === 1;
            nextBtn.disabled = currentPage === totalPages;
        }
        
        displayFiles();
        updateStats();
    }

    function updateStats() {
        const statsElement = document.getElementById('fileStats');
        const totalSize = filteredFiles.reduce((acc, file) => acc + file.size, 0);
        statsElement.innerHTML = `
            <div>
                <i class="fas fa-file-alt"></i> Total fichiers: ${filteredFiles.length}
            </div>
            <div>
                <i class="fas fa-database"></i> Taille totale: ${formatFileSize(totalSize)}
            </div>
        `;
    }

    function formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    function displayFiles() {
        const start = (currentPage - 1) * ITEMS_PER_PAGE;
        const end = start + ITEMS_PER_PAGE;
        const filesForPage = filteredFiles.slice(start, end);
        
        filesList.innerHTML = '';
        if (filesForPage.length === 0) {
            filesList.innerHTML = '<div class="no-files">Aucun fichier trouvé</div>';
            return;
        }
        
        filesForPage.forEach(file => addFileToList(file));
    }

    prevBtn.addEventListener('click', () => {
        if (currentPage > 1) {
            currentPage--;
            updatePagination();
        }
    });

    nextBtn.addEventListener('click', () => {
        const totalPages = Math.ceil(filteredFiles.length / ITEMS_PER_PAGE);
        if (currentPage < totalPages) {
            currentPage++;
            updatePagination();
        }
    });

    // Gestion du drag & drop
    dropZone.addEventListener('dragover', (e) => {
        e.preventDefault();
        dropZone.classList.add('drag-active');
    });

    dropZone.addEventListener('dragleave', () => {
        dropZone.classList.remove('drag-active');
    });

    dropZone.addEventListener('drop', (e) => {
        e.preventDefault();
        dropZone.classList.remove('drag-active');
        const files = Array.from(e.dataTransfer.files);
        if (files.length > 0) {
            files.forEach(file => uploadFile(file));
        }
    });

    // Gestion de l'input file
    fileInput.addEventListener('change', (e) => {
        const files = Array.from(e.target.files || []);
        if (files.length > 0) {
            files.forEach(file => uploadFile(file));
        }
    });

    // Amélioration de loadFiles avec meilleure gestion des erreurs
    async function loadFiles() {
        try {
            const response = await fetch('http://localhost:8082/api/files');
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const data = await response.json();
            console.log('Loaded files:', data); // Debug

            // Mise à jour des données
            allFiles = Array.isArray(data.files) ? data.files : [];
            refreshFilesList();
        } catch (error) {
            console.error('Error loading files:', error);
            showNotification('Erreur lors du chargement des fichiers', 'error');
        }
    }

    // Fonction de rafraîchissement centralisée
    function refreshFilesList() {
        // Mise à jour des fichiers filtrés
        filterAndSortFiles();
        // Réinitialiser la page si nécessaire
        const totalPages = Math.ceil(filteredFiles.length / ITEMS_PER_PAGE);
        if (currentPage > totalPages) {
            currentPage = Math.max(1, totalPages);
        }
        // Mettre à jour l'UI
        updatePagination();
        displayFiles();
        updateStats();
    }

    loadFiles();

    // Amélioration de uploadFile
    async function uploadFile(file) {
        try {
            const formData = new FormData();
            formData.append('file', file);
            
            const response = await fetch('http://localhost:8082/api/upload', {
                method: 'POST',
                body: formData
            });
            
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const result = await response.json();
            console.log('Upload result:', result); // Debug

            // Ajouter le nouveau fichier au tableau local
            if (result.files && result.files.length > 0) {
                const newFile = result.files[0];
                // Ajouter le nouveau fichier à allFiles
                allFiles.push(newFile);

                // Mettre à jour la vue
                refreshFilesList();
                
                showNotification(`Le fichier ${file.name} a été uploadé avec succès`);
            } else {
                // Fallback : recharger toute la liste si le serveur ne renvoie pas le nouveau fichier
                await loadFiles();
            }
        } catch (error) {
            console.error('Upload error:', error);
            showNotification(`Erreur lors de l'upload de ${file.name}`, 'error');
        }
    }

    // Amélioration de deleteFile
    window.deleteFile = async (fileId) => {
        try {
            const response = await fetch(`http://localhost:8082/api/files/${fileId}`, {
                method: 'DELETE'
            });
            const rep_json = await response.json();
            console.log('Delete result:', rep_json); // Debug

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }


            // Supprimer le fichier du tableau local
            allFiles = allFiles.filter(f => +f.id !== +fileId);
            
            // Mettre à jour la vue
            refreshFilesList();
            
            showNotification('Fichier supprimé avec succès');
        } catch (error) {
            console.error('Delete error:', error);
            showNotification('Erreur lors de la suppression', 'error');
        }
    };

    function getFileIcon(fileName) {
        const extension = fileName.split('.').pop().toLowerCase();
        const icons = {
            // Documents
            pdf: '<i class="far fa-file-pdf"></i>',
            doc: '<i class="far fa-file-word"></i>',
            docx: '<i class="far fa-file-word"></i>',
            txt: '<i class="far fa-file-alt"></i>',
            rtf: '<i class="far fa-file-alt"></i>',
            
            // Tableurs
            xls: '<i class="far fa-file-excel"></i>',
            xlsx: '<i class="far fa-file-excel"></i>',
            csv: '<i class="far fa-file-excel"></i>',
            
            // Présentations
            ppt: '<i class="far fa-file-powerpoint"></i>',
            pptx: '<i class="far fa-file-powerpoint"></i>',
            
            // Images
            jpg: '<i class="far fa-file-image"></i>',
            jpeg: '<i class="far fa-file-image"></i>',
            png: '<i class="far fa-file-image"></i>',
            gif: '<i class="far fa-file-image"></i>',
            svg: '<i class="far fa-file-image"></i>',
            
            // Audio
            mp3: '<i class="far fa-file-audio"></i>',
            wav: '<i class="far fa-file-audio"></i>',
            ogg: '<i class="far fa-file-audio"></i>',
            
            // Vidéo
            mp4: '<i class="far fa-file-video"></i>',
            avi: '<i class="far fa-file-video"></i>',
            mov: '<i class="far fa-file-video"></i>',
            
            // Archives
            zip: '<i class="far fa-file-archive"></i>',
            rar: '<i class="far fa-file-archive"></i>',
            '7z': '<i class="far fa-file-archive"></i>',
            
            // Code
            html: '<i class="far fa-file-code"></i>',
            css: '<i class="far fa-file-code"></i>',
            js: '<i class="far fa-file-code"></i>',
            json: '<i class="far fa-file-code"></i>',
            php: '<i class="far fa-file-code"></i>',
            py: '<i class="far fa-file-code"></i>'
        };
        return icons[extension] || '<i class="far fa-file"></i>';
    }

    function addFileToList(file) {
        const fileElement = document.createElement('div');
        fileElement.id = `file-${file.id}`;
        fileElement.className = 'file-item';
        
        const fileIcon = getFileIcon(file.name);
        const fileDate = new Date(file.uploadDate).toLocaleDateString();
        
        fileElement.innerHTML = `
            <div class="file-info">
                <span class="file-icon">${fileIcon}</span>
                <div>
                    <p class="file-name">${file.name}</p>
                    <p class="file-details">
                        <span class="file-size">
                            <i class="fas fa-weight-hanging"></i> ${formatFileSize(file.size)}
                        </span>
                        <span class="file-date">
                            <i class="far fa-calendar-alt"></i> ${fileDate}
                        </span>
                    </p>
                </div>
            </div>
            <div class="file-actions">
                <button class="preview-btn" onclick="previewFile('${file.id}')" title="Aperçu">
                    <i class="far fa-eye"></i>
                </button>
                <button class="download-btn" onclick="downloadFile('${file.id}')" title="Télécharger">
                    <i class="fas fa-download"></i>
                </button>
                <button class="delete-btn" onclick="deleteFile('${file.id}')" title="Supprimer">
                    <i class="far fa-trash-alt"></i>
                </button>
            </div>
        `;
        filesList.appendChild(fileElement);
    }

    function showNotification(message, type = 'success') {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        
        const icon = type === 'success' 
            ? '<i class="fas fa-check-circle"></i>' 
            : '<i class="fas fa-exclamation-circle"></i>';
            
        notification.innerHTML = `
            <span class="notification-icon">${icon}</span>
            <span class="notification-message">${message}</span>
        `;
        
        notificationsContainer.appendChild(notification);
        
        setTimeout(() => {
            notification.remove();
        }, 3000);
    }

    window.previewFile = (fileId) => {
        const file = allFiles.find(f => +f.id === +fileId);
                
        if (!file) return;

        const previewModal = document.createElement('div');
        previewModal.className = 'preview-modal';
        previewModal.innerHTML = `
            <div class="preview-content">
                <div class="preview-header">
                    <h3>${getFileIcon(file.name)} ${file.name}</h3>
                    <button onclick="this.closest('.preview-modal').remove()">
                        <i class="fas fa-times"></i>
                    </button>
                </div>
                <div class="preview-body">
                    <div class="preview-info">
                        <p>
                            <i class="fas fa-weight-hanging"></i>
                            <strong>Taille:</strong> ${formatFileSize(file.size)}
                        </p>
                        <p>
                            <i class="far fa-calendar-alt"></i>
                            <strong>Date:</strong> ${new Date(file.uploadDate).toLocaleString()}
                        </p>
                    </div>
                    <div class="preview-actions">
                        <button onclick="downloadFile('${file.id}')">
                            <i class="fas fa-download"></i> Télécharger
                        </button>
                    </div>
                </div>
            </div>
        `;
        document.body.appendChild(previewModal);
    };

    window.downloadFile = (fileId) => {
        const file = allFiles.find(f => f.id === fileId);
        if (!file) return;
        
        showNotification(`Téléchargement de ${file.name} commencé`);
    };

    // Chargement initial
    document.addEventListener('DOMContentLoaded', loadFiles);
});