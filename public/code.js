// This is already in your code.js, but make sure it's there:
document.addEventListener('DOMContentLoaded', function() {
    // Fetch leagues immediately when the page loads
    fetchLeagues();

    // Add event listener to the calculate button
    document.getElementById('calculate').addEventListener('click', function() {
        mainRender();
    });

    // Add event listener to league select to save selection
    document.getElementById('league-select').addEventListener('change', function() {
        localStorage.setItem('selectedLeague', this.value);
        updateLeagueDisplay(this.value);
    });
});

// Fetch available leagues and populate the selector
function fetchLeagues() {
    fetch('/api/leagues')
        .then(response => response.json())
        .then(data => {
            const leagueSelect = document.getElementById('league-select');
            leagueSelect.innerHTML = ''; // Clear loading option

            if (data.leagues && data.leagues.length > 0) {
                // Sort leagues - put trade leagues first, then SSF, then hardcore
                const sortedLeagues = data.leagues.sort((a, b) => {
                    // Put current leagues before old leagues
                    if (a.indexed !== b.indexed) return b.indexed - a.indexed;

                    // Non-hardcore before hardcore
                    if (a.hardcore !== b.hardcore) return a.hardcore - b.hardcore;

                    // Alphabetical order for the rest
                    return a.name.localeCompare(b.name);
                });

                sortedLeagues.forEach(league => {
                    const option = document.createElement('option');
                    option.value = league.name;
                    option.textContent = league.displayName || league.name;
                    leagueSelect.appendChild(option);
                });

                // Select the previously selected league or the first league
                const savedLeague = localStorage.getItem('selectedLeague');
                if (savedLeague && leagueSelect.querySelector(`option[value="${savedLeague}"]`)) {
                    leagueSelect.value = savedLeague;
                } else {
                    // Default to the first indexed league or first in the list
                    const defaultLeague = sortedLeagues.find(l => l.indexed) || sortedLeagues[0];
                    leagueSelect.value = defaultLeague.name;
                    localStorage.setItem('selectedLeague', defaultLeague.name);
                }

                // Update the league display
                updateLeagueDisplay(leagueSelect.value);
            } else {
                // Fallback to Standard if no leagues are returned
                const option = document.createElement('option');
                option.value = 'Standard';
                option.textContent = 'Standard';
                leagueSelect.appendChild(option);
                updateLeagueDisplay('Standard');
            }
        })
        .catch(error => {
            console.error('Error fetching leagues:', error);
            const leagueSelect = document.getElementById('league-select');
            leagueSelect.innerHTML = '<option value="Standard">Standard</option>';
            updateLeagueDisplay('Standard');
        });
}

// Update the league display in the results section
function updateLeagueDisplay(leagueName) {
    const leagueDisplay = document.getElementById('current-league');
    if (leagueDisplay) {
        leagueDisplay.textContent = `Current league: ${leagueName}`;
    }
}

// Main render function to calculate gem values
function mainRender() {
    let ignoreAfterChaosValue = document.querySelector("#ignoreAfterChaos").value;
    const selectedLeague = document.getElementById('league-select').value || 'Standard';

    // Update UI to show loading state
    document.getElementById("red-result").textContent = "Red Gems: Calculating...";
    document.getElementById("green-result").textContent = "Green Gems: Calculating...";
    document.getElementById("blue-result").textContent = "Blue Gems: Calculating...";

    fetch(`/api/skill-gems?league=${encodeURIComponent(selectedLeague)}`)
        .then((response) => response.json())
        .then((data) => {
            let checkedGemLevel = document.querySelector(
                "#gemLevel input[type=radio]:checked"
            ).value;
            let checkedGemQuality = document.querySelector(
                "#gemQuality input[type=radio]:checked"
            ).value;

            let redGems = [];
            let greenGems = [];
            let blueGems = [];

            data.lines.forEach((element) => {
                if (element.tradeFilter !== undefined &&
                    element.corrupted == (checkedGemLevel > 20 || checkedGemQuality > 20 ? true : undefined) &&
                    element.gemLevel == checkedGemLevel &&
                    element.gemQuality == (checkedGemQuality > 0 ? checkedGemQuality : undefined)) {

                    if (red.includes(element.name)) {
                        redGems.push(element.chaosValue || 0);
                    } else if (green.includes(element.name)) {
                        greenGems.push(element.chaosValue || 0);
                    } else if (blue.includes(element.name)) {
                        blueGems.push(element.chaosValue || 0);
                    }
                }
            });

            redGems.sort((a, b) => b - a);
            greenGems.sort((a, b) => b - a);
            blueGems.sort((a, b) => b - a);

            let redProbabilities = calculateProbability(redGems.length);
            let greenProbabilities = calculateProbability(greenGems.length);
            let blueProbabilities = calculateProbability(blueGems.length);

            let redROI = calculateROI(redProbabilities, redGems, ignoreAfterChaosValue);
            let greenROI = calculateROI(greenProbabilities, greenGems, ignoreAfterChaosValue);
            let blueROI = calculateROI(blueProbabilities, blueGems, ignoreAfterChaosValue);

            document.getElementById("red-result").textContent =
                `Red Gems: Expected ROI = ${redROI.toFixed(2)} chaos`;
            document.getElementById("green-result").textContent =
                `Green Gems: Expected ROI = ${greenROI.toFixed(2)} chaos`;
            document.getElementById("blue-result").textContent =
                `Blue Gems: Expected ROI = ${blueROI.toFixed(2)} chaos`;
        })
        .catch((error) => {
            console.error("Error fetching data:", error);
            document.getElementById("red-result").textContent = "Error fetching data.";
            document.getElementById("green-result").textContent = "Error fetching data.";
            document.getElementById("blue-result").textContent = "Error fetching data.";
        });
}