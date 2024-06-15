
import BlockHeight from './components/BlockHeight';
import BitcoinPriceChart from './components/BitcoinPriceChart';
import './App.css'; // Ensure you have appropriate styles

const App = () => {
  return (
    <div className="container">
      <h1>Block Explorer </h1>
      <BlockHeight />
      <BitcoinPriceChart />
    </div>
  );
};

export default App;
