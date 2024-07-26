const express = require("express");
const axios = require("axios");
const fs = require("fs");
const app = express();
const port = 3030;
const cacheFilePath = "./cache/skillGems.json";
const cacheDuration = 30 * 60 * 1000; // 30 minutes in milliseconds

app.use(express.static("public"));

// Function to get current timestamp
const getCurrentTimestamp = () => new Date().getTime();

app.get("/api/skill-gems", async (req, res) => {
  try {
    // Check if cache file exists and is still valid
    if (fs.existsSync(cacheFilePath)) {
      const stats = fs.statSync(cacheFilePath);
      const fileAge = getCurrentTimestamp() - stats.mtimeMs;

      if (fileAge < cacheDuration) {
        const cachedData = fs.readFileSync(cacheFilePath);
        console.log("Using cached data");
        return res.json(JSON.parse(cachedData));
      }
    }

    // If cache is old or doesn't exist, fetch new data
    console.log("Fetching new data");
    const response = await axios.get(
      "https://poe.ninja/api/data/itemoverview?league=Standard&type=SkillGem"
    );

    // Save the new data to cache
    fs.writeFileSync(cacheFilePath, JSON.stringify(response.data));

    res.json(response.data);
  } catch (error) {
    console.log(error);
    res.status(500).send("Error fetching data");
  }
});

// Root should return an HTML file
app.get("/", (req, res) => {
  res.sendFile(__dirname + "/public/index.html");
});

app.listen(port, () => {
  console.log(`Server running at https://127.0.0.1:${port}`);
});
