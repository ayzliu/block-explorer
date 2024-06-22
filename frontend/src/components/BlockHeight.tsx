import useWebSocket from '../hooks/useWebSocket';
import '../BlockHeight.css'; 




const BlockHeight = () => {
  const data = useWebSocket('ws://127.0.0.1:8080');
  const filteredData = data.filter(d => d.height !== 0);
  const latestBlocks = filteredData.slice(-1).reverse(); // Get the latest block

  return (
    <div className="block-height">
      <h2>Latest Blocks</h2>
      <table>
        <thead>
          <tr>
            <th>Height</th>
            <th>Timestamp</th>
          </tr>
        </thead>
        <tbody>
          {latestBlocks.map((block, index) => (
            <tr key={index}>
              <td>{block.height}</td>
              <td>{new Date(block.timestamp * 1000).toLocaleString()}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default BlockHeight;