import { useEffect, useState } from 'react';

interface DataPayload {
  height: number;
  price: number;
  timestamp: number;
}

const useWebSocket = (url: string) => {
  const [data, setData] = useState<DataPayload[]>([]);

  useEffect(() => {
    const ws = new WebSocket(url);

    ws.onopen = () => {
      console.log('WebSocket connection opened');
      ws.send('subscribe');
    };

    ws.onmessage = (event) => {
      console.log('Received:', event.data);
      const payload: DataPayload = JSON.parse(event.data);
      setData((prevData) => [...prevData, payload]);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('WebSocket connection closed');
    };

    return () => {
      console.log('Deactivating WebSocket connection...');
      ws.close();
    };
  }, [url]);

  return data;
};

export default useWebSocket;