document.addEventListener('DOMContentLoaded', () => {
    const dropZone = document.getElementById('dropZone');
    const fileInput = document.getElementById('fileInput');
    const filesList = document.getElementById('filesList');
    
    // Charger les fichiers au démarrage
    loadFiles();

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
        const file = e.dataTransfer.files[0];
        if (file) uploadFile(file);
    });

    // Gestion de l'input file
    fileInput.addEventListener('change', (e) => {
        const file = e.target.files[0];
        if (file) uploadFile(file);
    });

    // Charger la liste des fichiers
    async function loadFiles() {
        try {
            const response = await fetch('http://localhost:8082/api/files');
            const data = await response.json();
            filesList.innerHTML = '';
            data.files.forEach(file => {
                addFileToList(file)
            });
        } catch (error) {
            console.error('Erreur chargement:', error);
        }
    }

    // Upload d'un fichier
    async function uploadFile(file) {
        try {
            const formData = new FormData();
            formData.append('file', file);

            console.log("Form DATA: ", formData)
            
            const response = await fetch('http://localhost:8082/api/upload', {
                method: 'POST',
                body: formData
            });
            
            const result = await response.json();
            console.log(result)
            addFileToList(result);
        } catch (error) {
           console.error('Erreur upload:', error);
        }
    }

    // Suppression d'un fichier
    window.deleteFile = async (fileId) => {
        try {
            await fetch(`http://localhost:3000/api/files/${fileId}`, {
                method: 'DELETE'
            });
            
            const fileElement = document.getElementById(`file-${fileId}`);
            if (fileElement) fileElement.remove();
        } catch (error) {
            console.error('Erreur suppression:', error);
        }
    };

    // Ajouter un fichier à la liste
    function addFileToList(file) {
        const fileElement = document.createElement('div');
        fileElement.id = `file-${file.id}`;
        fileElement.className = 'file-item';
        fileElement.innerHTML = `
            <div class="file-info">
                <span>📄</span>
                <div>
                    <p class="file-name">${file.name}</p>
                    <p class="file-size">${(file.size / 1024).toFixed(2)} KB</p>
                </div>
            </div>
            <button class="delete-btn" onclick="deleteFile('${file.id}')">🗑️</button>
        `;
        filesList.appendChild(fileElement);
    }
});