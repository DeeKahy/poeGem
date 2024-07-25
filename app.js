const express = require('express');
const axios = require('axios');
const app = express();
const port = 3002;

app.get('/api/skill-gems', async (req, res) => {
    try {
        const response = await axios.get('https://poe.ninja/api/data/itemoverview?league=Necropolis&type=SkillGem');
        res.json(response.data);
    } catch (error) {
        res.status(500).send('Error fetching data');
    }
});
//root should return a html file
app.get('/', (req, res) => {
    res.sendFile(__dirname + '/index.html');
});

app.listen(port, () => {
    console.log(`Server running at http://0.0.0.0:${port}`);
});
