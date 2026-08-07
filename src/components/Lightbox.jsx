import './Lightbox.css';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function Lightbox({ item, onClose, onDelete, triggerToast }) {
    const [base64, setBase64] = useState('');
    const [confirmDelete, setConfirmDelete] = useState(false);
    
    useEffect(() => {
        invoke('get_thumbnail', { path: item.path }).then(res => {
            setBase64(res);
        }).catch(e => {
            console.error(e);
            triggerToast('Failed to load image');
        });
    }, [item.path]);

    useEffect(() => {
        let timer;
        if (confirmDelete) {
            timer = setTimeout(() => setConfirmDelete(false), 3000);
        }
        return () => clearTimeout(timer);
    }, [confirmDelete]);

    const handleOpenExt = async () => {
        try {
            await invoke('open_file_externally', { path: item.path });
        } catch (e) {
            triggerToast('Failed to open file: ' + e);
        }
    };

    const handleDeleteClick = () => {
        if (!confirmDelete) {
            setConfirmDelete(true);
        } else {
            onDelete();
        }
    };

    return (
        <div className="lightbox open">
            <div className="lightbox-header">
                <button className="icon-button transparent-btn" onClick={onClose}>
                    <span className="material-symbols-outlined">arrow_back</span>
                </button>
                <span className="lightbox-title">{item.filename}</span>
                <div className="lightbox-actions">
                    <button className="icon-button transparent-btn" onClick={handleOpenExt} title="Open externally">
                        <span className="material-symbols-outlined">open_in_new</span>
                    </button>
                    <button 
                        className={`icon-button transparent-btn ${confirmDelete ? 'confirm-delete' : ''}`} 
                        onClick={handleDeleteClick} 
                        title="Delete"
                    >
                        <span className="material-symbols-outlined">
                            {confirmDelete ? 'delete_forever' : 'delete'}
                        </span>
                    </button>
                </div>
            </div>
            <div className="lightbox-body">
                {base64 && <img className="loaded" src={`data:image/jpeg;base64,${base64}`} alt={item.filename} />}
            </div>
        </div>
    );
}
