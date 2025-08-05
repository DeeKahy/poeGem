// Main application JavaScript for POE Gem Calculator
document.addEventListener('DOMContentLoaded', function() {
    // Initialize the application
    initializeApp();
});

let currentLeagues = [];
let currentSkillGems = null;

async function initializeApp() {
    try {
        // Load leagues on startup
        await fetchAndPopulateLeagues();

        // Set up event listeners
        setupEventListeners();

        console.log('Application initialized successfully');
    } catch (error) {
        console.error('Failed to initialize application:', error);
        showError('Failed to initialize application. Please refresh the page.');
    }
}

function setupEventListeners() {
    // Calculate button
    const calculateBtn = document.getElementById('calculate');
    if (calculateBtn) {
        calculateBtn.addEventListener('click', performCalculation);
    }

    // League selector
    const leagueSelect = document.getElementById('league-select');
    if (leagueSelect) {
        leagueSelect.addEventListener('change', function() {
            localStorage.setItem('selectedLeague', this.value);
            console.log('League changed to:', this.value);
        });
    }

    // Form inputs for real-time updates
    const formInputs = document.querySelectorAll('#value-form input, #value-form select');
    formInputs.forEach(input => {
        input.addEventListener('change', function() {
            console.log('Form input changed:', this.name, this.value);
        });
    });
}

async function fetchAndPopulateLeagues() {
    try {
        console.log('Fetching leagues from API...');
        const response = await fetch('/api/leagues');

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();
        currentLeagues = data.leagues || [];

        populateLeagueSelector(currentLeagues);
        console.log(`Loaded ${currentLeagues.length} leagues`);
    } catch (error) {
        console.error('Error fetching leagues:', error);

        // Fallback to Standard league
        const leagueSelect = document.getElementById('league-select');
        if (leagueSelect) {
            leagueSelect.innerHTML = '<option value="Standard">Standard (Fallback)</option>';
        }

        showError('Failed to load leagues. Using Standard as fallback.');
    }
}

function populateLeagueSelector(leagues) {
    const leagueSelect = document.getElementById('league-select');
    if (!leagueSelect) return;

    // Clear existing options
    leagueSelect.innerHTML = '';

    if (leagues.length === 0) {
        leagueSelect.innerHTML = '<option value="Standard">Standard</option>';
        return;
    }

    // Sort leagues - current leagues first, then by type
    const sortedLeagues = leagues.sort((a, b) => {
        // Current leagues before old leagues
        if (a.indexed !== b.indexed) return b.indexed - a.indexed;

        // Non-hardcore before hardcore
        if (a.hardcore !== b.hardcore) return a.hardcore - b.hardcore;

        // Alphabetical order
        return a.name.localeCompare(b.name);
    });

    // Add options
    sortedLeagues.forEach(league => {
        const option = document.createElement('option');
        option.value = league.name;
        option.textContent = league.displayName || league.name;

        // Add indicators for league type
        if (league.hardcore) {
            option.textContent += ' (HC)';
        }
        if (!league.indexed) {
            option.textContent += ' (Old)';
        }

        leagueSelect.appendChild(option);
    });

    // Restore saved league or select default
    const savedLeague = localStorage.getItem('selectedLeague');
    if (savedLeague && leagueSelect.querySelector(`option[value="${savedLeague}"]`)) {
        leagueSelect.value = savedLeague;
    } else {
        // Default to first indexed league
        const defaultLeague = sortedLeagues.find(l => l.indexed) || sortedLeagues[0];
        if (defaultLeague) {
            leagueSelect.value = defaultLeague.name;
            localStorage.setItem('selectedLeague', defaultLeague.name);
        }
    }
}

async function performCalculation() {
    try {
        // Update UI to show loading state
        setCalculationLoadingState(true);

        // Get form values
        const formData = getFormData();
        console.log('Starting calculation with parameters:', formData);

        // Build query parameters
        const params = new URLSearchParams();
        if (formData.league) params.append('league', formData.league);
        if (formData.ignoreAfterChaos !== null) params.append('ignore_after_chaos', formData.ignoreAfterChaos);
        if (formData.gemLevel !== null) params.append('gem_level', formData.gemLevel);
        if (formData.gemQuality !== null) params.append('gem_quality', formData.gemQuality);

        // Make API call
        const response = await fetch(`/api/calculate?${params}`);

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const result = await response.json();
        console.log('Calculation result:', result);

        // Update UI with results
        displayCalculationResults(result);

    } catch (error) {
        console.error('Calculation failed:', error);
        showError('Calculation failed. Please try again.');
        displayCalculationError();
    } finally {
        setCalculationLoadingState(false);
    }
}

function getFormData() {
    const leagueSelect = document.getElementById('league-select');
    const ignoreAfterChaos = document.getElementById('ignoreAfterChaos');
    const gemLevelRadio = document.querySelector('#gemLevel input[type=radio]:checked');
    const gemQualityRadio = document.querySelector('#gemQuality input[type=radio]:checked');

    return {
        league: leagueSelect ? leagueSelect.value : 'Standard',
        ignoreAfterChaos: ignoreAfterChaos ? parseFloat(ignoreAfterChaos.value) || 0 : 5,
        gemLevel: gemLevelRadio ? parseInt(gemLevelRadio.value) : 1,
        gemQuality: gemQualityRadio ? parseInt(gemQualityRadio.value) : 0
    };
}

function setCalculationLoadingState(isLoading) {
    const calculateBtn = document.getElementById('calculate');
    const resultElements = [
        document.getElementById('red-result'),
        document.getElementById('green-result'),
        document.getElementById('blue-result')
    ];

    if (calculateBtn) {
        calculateBtn.disabled = isLoading;
        calculateBtn.textContent = isLoading ? 'Calculating...' : 'Calculate';
    }

    if (isLoading) {
        resultElements.forEach((element, index) => {
            if (element) {
                const colors = ['Red', 'Green', 'Blue'];
                element.textContent = `${colors[index]} Gems: Calculating...`;
            }
        });
    }
}

function displayCalculationResults(result) {
    const redResult = document.getElementById('red-result');
    const greenResult = document.getElementById('green-result');
    const blueResult = document.getElementById('blue-result');

    if (redResult) {
        redResult.textContent = `Red Gems: Expected ROI = ${result.red_roi.toFixed(2)} chaos`;
    }

    if (greenResult) {
        greenResult.textContent = `Green Gems: Expected ROI = ${result.green_roi.toFixed(2)} chaos`;
    }

    if (blueResult) {
        blueResult.textContent = `Blue Gems: Expected ROI = ${result.blue_roi.toFixed(2)} chaos`;
    }

    // Highlight the best option
    highlightBestROI(result);

    console.log('Results displayed successfully');
}

function highlightBestROI(result) {
    // Remove existing highlighting
    const resultElements = [
        document.getElementById('red-result'),
        document.getElementById('green-result'),
        document.getElementById('blue-result')
    ];

    resultElements.forEach(element => {
        if (element) {
            element.style.fontWeight = 'normal';
            element.style.color = '';
        }
    });

    // Find and highlight the best ROI
    const rois = [
        { element: resultElements[0], value: result.red_roi, name: 'red' },
        { element: resultElements[1], value: result.green_roi, name: 'green' },
        { element: resultElements[2], value: result.blue_roi, name: 'blue' }
    ];

    const bestROI = rois.reduce((best, current) =>
        current.value > best.value ? current : best
    );

    if (bestROI.element && bestROI.value > 0) {
        bestROI.element.style.fontWeight = 'bold';
        bestROI.element.style.color = '#28a745'; // Bootstrap success color
        console.log(`Best ROI: ${bestROI.name} with ${bestROI.value.toFixed(2)} chaos`);
    }
}

function displayCalculationError() {
    const resultElements = [
        document.getElementById('red-result'),
        document.getElementById('green-result'),
        document.getElementById('blue-result')
    ];

    resultElements.forEach((element, index) => {
        if (element) {
            const colors = ['Red', 'Green', 'Blue'];
            element.textContent = `${colors[index]} Gems: Error calculating ROI`;
            element.style.color = '#dc3545'; // Bootstrap danger color
        }
    });
}

function showError(message) {
    console.error('Application error:', message);

    // You could implement a toast notification system here
    // For now, we'll just log to console and could show an alert
    if (typeof window !== 'undefined' && window.confirm) {
        // Only show alert if we're in a browser environment
        // alert(message);
    }
}

// Utility function for debugging
function debugState() {
    console.log('=== Application Debug State ===');
    console.log('Current leagues:', currentLeagues);
    console.log('Form data:', getFormData());
    console.log('Selected league:', localStorage.getItem('selectedLeague'));
    console.log('==============================');
}

// Export for debugging (if in module environment)
if (typeof window !== 'undefined') {
    window.debugPOEApp = debugState;
}
