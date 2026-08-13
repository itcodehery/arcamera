import './CameraView.css';
import { useEffect, useRef, useState } from 'react';

const ASPECT_MAP = {
    '4:3': 4 / 3,
    '1:1': 1,
    '16:9': 16 / 9,
};

export default function CameraView({ mirror, grid, aspectRatio, flashEnabled }) {
    const imgRef = useRef(null);
    const [flash, setFlash] = useState(false);

    useEffect(() => {
        let active = true;
        let lastUrl = null;

        const updateFrame = async () => {
            if (!active) return;
            try {
                const response = await fetch("http://127.0.0.1:49283/frame");
                if (response.ok) {
                    const blob = await response.blob();
                    const url = URL.createObjectURL(blob);
                    
                    if (imgRef.current) {
                        imgRef.current.src = url;
                    }
                    
                    // Revoke the old URL *after* assigning the new one to prevent flickering
                    if (lastUrl) {
                        URL.revokeObjectURL(lastUrl);
                    }
                    lastUrl = url;
                }
            } catch (err) {
                // Ignore fetch errors, likely just the server restarting
            }
            
            // Loop instantly for 30fps
            if (active) {
                requestAnimationFrame(updateFrame);
            }
        };

        // Start the loop
        updateFrame();
        
        const handleFlash = () => {
            if (flashEnabled) {
                setFlash(true);
                setTimeout(() => setFlash(false), 3000);
            }
        };
        
        window.addEventListener('arcamera-flash', handleFlash);
        return () => {
            active = false;
            if (lastUrl) URL.revokeObjectURL(lastUrl);
            window.removeEventListener('arcamera-flash', handleFlash);
        };
    }, [flashEnabled]);

    const ratio = ASPECT_MAP[aspectRatio] || 4 / 3;

    // Use max-width to constrain the viewport shape based on aspect ratio
    const containerStyle = {
        maxWidth: `calc((100vh - 280px) * ${ratio})`,
    };

    return (
        <div className="camera-wrapper">
            <div className="camera-container" style={containerStyle}>
                <img 
                    ref={imgRef}
                    id="camera-feed" 
                    style={{ transform: mirror ? 'translate(-50%, -50%) scaleX(-1)' : 'translate(-50%, -50%)' }}
                    alt="Camera Feed"
                />
                {grid && (
                    <div className="grid-overlay" style={{ display: 'block' }}>
                        <div className="grid-line grid-h" style={{ top: '33.33%' }}></div>
                        <div className="grid-line grid-h" style={{ top: '66.66%' }}></div>
                        <div className="grid-line grid-v" style={{ left: '33.33%' }}></div>
                        <div className="grid-line grid-v" style={{ left: '66.66%' }}></div>
                    </div>
                )}
                
                {flash && (
                    <div className="flash-overlay"></div>
                )}
            </div>
        </div>
    );
}
