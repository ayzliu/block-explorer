import { useMemo } from 'react';
import '../BitcoinPriceChart.css';
import { Line } from 'react-chartjs-2';

import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
} from 'chart.js';
import useWebSocket from '../hooks/useWebSocket';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
);

const BitcoinPriceChart = () => {
  const data = useWebSocket('ws://127.0.0.1:8080');

  const filteredData = data.filter(d => d.price !== 0);

  const chartData = useMemo(() => ({
    labels: filteredData.map(d => new Date(d.timestamp * 1000).toLocaleTimeString()),
    datasets: [
      {
        label: 'Bitcoin Price (USD)',
        data: filteredData.map(d => d.price),
        borderColor: 'rgba(75,192,192,1)',
        fill: false,
      },
    ],
  }), [filteredData]);

  return (
    <div className="bitcoin-price-chart">
      <h2>Bitcoin Price</h2>
      <div className="chart-container">
        <Line data={chartData} />
      </div>
    </div>
  );
};

export default BitcoinPriceChart;
