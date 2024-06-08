import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './App.css';

interface Block {
    height: number;
    timestamp: string;
}

const App: React.FC = () => {
    const [blocks, setBlocks] = useState<Block[]>([]);
    const [isLoading, setIsLoading] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchBlocks = async () => {
            setIsLoading(true);
            try {
                const response = await axios.get('http://localhost:3001/blocks');
                setBlocks(response.data);
                setError(null);
            } catch (error) {
                console.error('Error fetching blocks:', error);
                setError('Failed to fetch blocks');
            } finally {
                setIsLoading(false);
            }
        };

        fetchBlocks();
        const interval = setInterval(fetchBlocks, 60000); // Fetch every minute
        return () => clearInterval(interval); // Cleanup on component unmount
    }, []);

    const formatTimestamp = (timestamp: string) => {
        const timeDiff = Math.floor((Date.now() - new Date(timestamp).getTime()) / 1000);
        if (timeDiff < 60) return `${timeDiff} secs ago`;
        else if (timeDiff < 3600) return `${Math.floor(timeDiff / 60)} mins ago`;
        else return `${Math.floor(timeDiff / 3600)} hours ago`;
    };

    if (isLoading) return <p>Loading...</p>;
    if (error) return <p>Error: {error}</p>;

    return (
        <div className="app">
            <h1>Bitcoin Block Explorer</h1>
            <div className="block-list">
                {blocks.map((block, index) => (
                    <div key={index} className="block-item">
                        <div className="block-icon">📦</div>
                        <div className="block-details">
                            <div className="block-height">{block.height}</div>
                            <div className="block-timestamp">{formatTimestamp(block.timestamp)}</div>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};

export default App;
