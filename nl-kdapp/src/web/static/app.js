// Natural Language kdapp Interface - Client Application

const API_BASE = '/api';

// DOM Elements
const promptInput = document.getElementById('prompt-input');
const generateBtn = document.getElementById('generate-btn');
const statusSection = document.getElementById('status-section');
const statusMessage = document.getElementById('status-message');
const resultSection = document.getElementById('result-section');
const errorSection = document.getElementById('error-section');
const errorMessage = document.getElementById('error-message');
const episodeId = document.getElementById('episode-id');
const shareLink = document.getElementById('share-link');
const copyBtn = document.getElementById('copy-btn');
const playBtn = document.getElementById('play-btn');
const tryAgainBtn = document.getElementById('try-again-btn');
const exampleBtns = document.querySelectorAll('.example-btn');

// Event Listeners
generateBtn.addEventListener('click', generateEpisode);
promptInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') generateEpisode();
});

copyBtn.addEventListener('click', copyShareLink);
playBtn.addEventListener('click', playGame);
tryAgainBtn.addEventListener('click', resetUI);

exampleBtns.forEach(btn => {
    btn.addEventListener('click', () => {
        promptInput.value = btn.dataset.prompt;
        promptInput.focus();
    });
});

// Functions
async function generateEpisode() {
    const prompt = promptInput.value.trim();
    if (!prompt) {
        alert('Please enter a prompt');
        return;
    }

    // Update UI
    generateBtn.disabled = true;
    hideAllSections();
    statusSection.classList.remove('hidden');
    updateStatus('Processing your request...');

    try {
        // Simulate status updates
        setTimeout(() => updateStatus('Understanding your prompt...'), 1000);
        setTimeout(() => updateStatus('Generating Episode code...'), 2000);
        setTimeout(() => updateStatus('Deploying to Kaspa network...'), 3000);

        const response = await fetch(`${API_BASE}/generate`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ prompt }),
        });

        const data = await response.json();

        if (data.status === 'success') {
            showResult(data.episode_id, data.share_link);
        } else {
            showError(data.error || 'Failed to generate Episode');
        }
    } catch (error) {
        showError('Network error. Please try again.');
        console.error('Error:', error);
    } finally {
        generateBtn.disabled = false;
    }
}

function updateStatus(message) {
    statusMessage.textContent = message;
}

function showResult(id, link) {
    hideAllSections();
    episodeId.textContent = id;
    shareLink.value = link;
    resultSection.classList.remove('hidden');
}

function showError(message) {
    hideAllSections();
    errorMessage.textContent = message;
    errorSection.classList.remove('hidden');
}

function hideAllSections() {
    statusSection.classList.add('hidden');
    resultSection.classList.add('hidden');
    errorSection.classList.add('hidden');
}

function resetUI() {
    hideAllSections();
    promptInput.value = '';
    promptInput.focus();
}

function copyShareLink() {
    shareLink.select();
    document.execCommand('copy');
    
    const originalText = copyBtn.textContent;
    copyBtn.textContent = 'Copied!';
    copyBtn.style.background = '#5cb85c';
    
    setTimeout(() => {
        copyBtn.textContent = originalText;
        copyBtn.style.background = '';
    }, 2000);
}

function playGame() {
    const link = shareLink.value;
    if (link) {
        window.open(link, '_blank');
    }
}

// WebSocket connection for real-time updates (future enhancement)
function connectWebSocket(episodeId) {
    const ws = new WebSocket(`ws://${window.location.host}/ws/${episodeId}`);
    
    ws.onopen = () => {
        console.log('WebSocket connected');
    };
    
    ws.onmessage = (event) => {
        console.log('WebSocket message:', event.data);
        // Handle real-time updates
    };
    
    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };
    
    ws.onclose = () => {
        console.log('WebSocket disconnected');
    };
    
    return ws;
}

// Initialize
promptInput.focus();