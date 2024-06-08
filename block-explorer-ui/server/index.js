const express = require('express');
const { Client } = require('pg');
const cors = require('cors');

const app = express();
const port = 3001;

const client = new Client({
    host: 'localhost',
    user: 'postgres',
    password: 'block',
    database: 'postgres',
});

client.connect();

app.use(cors());

app.get('/blocks', async (req, res) => {
    try {
        const result = await client.query('SELECT height, timestamp FROM blocks ORDER BY id DESC LIMIT 5');
        res.json(result.rows);
    } catch (err) {
        console.error(err);
        res.status(500).send('Server Error');
    }
});

app.listen(port, () => {
    console.log(`Server running on http://localhost:${port}`);
});

