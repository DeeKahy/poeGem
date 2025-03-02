// app.js
const express = require('express');
const axios = require('axios');
const fs = require('fs');
const path = require('path');
const app = express();
const PORT = process.env.PORT || 3000;

// Create cache directory if it doesn't exist
const cacheDir = path.join(__dirname, 'cache');
if (!fs.existsSync(cacheDir)) {
  fs.mkdirSync(cacheDir);
}

// Serve static files
app.use(express.static('public'));

// API endpoint to get available leagues
app.get('/api/leagues', async (req, res) => {
  try {
    const response = await axios.get('https://poe.ninja/api/data/getindexstate');

    // Extract economy leagues from the response
    const leagues = [
      ...response.data.economyLeagues || [],
      ...response.data.oldEconomyLeagues || []
    ];

    res.json({ leagues });
  } catch (error) {
    console.error('Error fetching leagues:', error);
    res.status(500).json({ error: 'Failed to fetch leagues' });
  }
});

// API endpoint to get skill gem data for a specific league
app.get('/api/skill-gems', async (req, res) => {
  const league = req.query.league || 'Standard'; // Default to Standard if no league specified

  try {
    const cacheFile = path.join(cacheDir, `skillGems_${league}.json`);
    let data;

    // Check if we have cached data less than 1 hour old
    if (fs.existsSync(cacheFile)) {
      const stats = fs.statSync(cacheFile);
      const fileAge = (new Date().getTime() - stats.mtime.getTime()) / 1000 / 60;

      if (fileAge < 60) { // Less than 1 hour old
        data = JSON.parse(fs.readFileSync(cacheFile, 'utf8'));
        return res.json(data);
      }
    }

    // Fetch fresh data
    const response = await axios.get(`https://poe.ninja/api/data/itemoverview?league=${encodeURIComponent(league)}&type=SkillGem&language=en`);
    data = response.data;

    // Cache the data
    fs.writeFileSync(cacheFile, JSON.stringify(data));

    res.json(data);
  } catch (error) {
    console.error('Error fetching skill gem data:', error);
    res.status(500).json({ error: 'Failed to fetch skill gem data' });
  }
});

app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});

// Add this new endpoint to app.js
app.get('/api/gem-colors', async (req, res) => {
  const league = req.query.league || 'Standard';

  try {
    const cacheFile = path.join(cacheDir, `gemColors_${league}.json`);
    let gemColors = { red: [], green: [], blue: [] };

    // Check if we have cached data less than 24 hours old
    if (fs.existsSync(cacheFile)) {
      const stats = fs.statSync(cacheFile);
      const fileAge = (new Date().getTime() - stats.mtime.getTime()) / 1000 / 60 / 60;

      if (fileAge < 24) { // Less than 24 hours old
        gemColors = JSON.parse(fs.readFileSync(cacheFile, 'utf8'));
        return res.json(gemColors);
      }
    }

    // Fetch skill gem data from poe.ninja
    const response = await axios.get(`https://poe.ninja/api/data/itemoverview?league=${encodeURIComponent(league)}&type=SkillGem&language=en`);

    // Process all gems to identify transfigured gems and their colors
    response.data.lines.forEach(gem => {
      // Check if it's a transfigured gem (they all have "of" in their name)
      if (gem.name && gem.name.includes(' of ')) {
        // Determine color based on gem tags or other properties
        if (gem.icon && gem.icon.includes('Red')) {
          gemColors.red.push(gem.name);
        } else if (gem.icon && gem.icon.includes('Green')) {
          gemColors.green.push(gem.name);
        } else if (gem.icon && gem.icon.includes('Blue')) {
          gemColors.blue.push(gem.name);
        }
      }
    });

    // If we couldn't determine colors from icons, use the base gem name
    if (gemColors.red.length === 0 && gemColors.green.length === 0 && gemColors.blue.length === 0) {
      // Fetch base gem data from PoE API or another source
      const baseGemsResponse = await axios.get('https://poedb.tw/us/api/gems');
      const baseGems = baseGemsResponse.data;

      // Map transfigured gems to their base gems and determine color
      response.data.lines.forEach(gem => {
        if (gem.name && gem.name.includes(' of ')) {
          const baseName = gem.name.split(' of ')[0];
          const baseGem = baseGems.find(bg => bg.name === baseName);

          if (baseGem) {
            if (baseGem.color === 'red') {
              gemColors.red.push(gem.name);
            } else if (baseGem.color === 'green') {
              gemColors.green.push(gem.name);
            } else if (baseGem.color === 'blue') {
              gemColors.blue.push(gem.name);
            }
          }
        }
      });
    }

    // Cache the data
    fs.writeFileSync(cacheFile, JSON.stringify(gemColors));

    res.json(gemColors);
  } catch (error) {
    console.error('Error fetching gem colors:', error);
    res.status(500).json({ error: 'Failed to fetch gem colors' });
  }
});
