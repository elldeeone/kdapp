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
const toggleCodeBtn = document.getElementById('toggle-code-btn');
const generatedCode = document.getElementById('generated-code');

// Event Listeners
generateBtn.addEventListener('click', generateEpisode);
promptInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') generateEpisode();
});

copyBtn.addEventListener('click', copyShareLink);
playBtn.addEventListener('click', playGame);
tryAgainBtn.addEventListener('click', resetUI);
toggleCodeBtn.addEventListener('click', toggleCode);

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
            showResult(data.episode_id, data.share_link, data.generated_code);
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

function showResult(id, link, code) {
    hideAllSections();
    episodeId.textContent = id;
    
    // Construct full URL from relative path
    const fullUrl = link.startsWith('http') ? link : `${window.location.origin}${link}`;
    shareLink.value = fullUrl;
    
    // Store the episode ID for the play button
    window.currentEpisodeId = id;
    
    if (code) {
        generatedCode.querySelector('code').textContent = code;
        toggleCodeBtn.style.display = 'block';
    } else {
        toggleCodeBtn.style.display = 'none';
    }
    
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

function toggleCode() {
    if (generatedCode.classList.contains('hidden')) {
        generatedCode.classList.remove('hidden');
        toggleCodeBtn.textContent = 'Hide Code';
    } else {
        generatedCode.classList.add('hidden');
        toggleCodeBtn.textContent = 'Show Code';
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
if (promptInput) {
    promptInput.focus();
}

// Dynamic app functionality for /app/:episode_id pages
function initializeApp() {
    if (!window.episodeId) return;
    
    const appContent = document.getElementById('app-content');
    const countdown = document.querySelector('.countdown');
    
    // Connect WebSocket for real-time updates
    const ws = connectAppWebSocket(window.episodeId);
    
    // Update countdown timer
    if (countdown) {
        updateCountdown(countdown);
        setInterval(() => updateCountdown(countdown), 60000); // Update every minute
    }
    
    // Load appropriate UI based on Episode type
    loadEpisodeUI(window.episodeType, appContent);
}

function connectAppWebSocket(episodeId) {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${protocol}//${window.location.host}/ws/${episodeId}`);
    
    ws.onopen = () => {
        console.log('Connected to Episode:', episodeId);
        ws.send(JSON.stringify({
            type: 'Subscribe',
            episode_id: episodeId,
            session_id: getSessionId()
        }));
    };
    
    ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        handleWebSocketMessage(msg);
    };
    
    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };
    
    ws.onclose = () => {
        console.log('Disconnected from Episode');
        // Try to reconnect after 3 seconds
        setTimeout(() => connectAppWebSocket(episodeId), 3000);
    };
    
    window.episodeWs = ws;
    return ws;
}

function handleWebSocketMessage(msg) {
    switch (msg.type) {
        case 'StateUpdate':
            updateEpisodeState(msg.state);
            break;
        case 'ParticipantUpdate':
            updateParticipantCount(msg.participant_count);
            break;
        case 'ExpirationWarning':
            showExpirationWarning(msg.remaining_seconds);
            break;
        case 'Error':
            showError(msg.message);
            break;
    }
}

function loadEpisodeUI(episodeType, container) {
    // Clear loading message
    container.innerHTML = '';
    
    // Load UI based on Episode type
    switch (episodeType.toLowerCase()) {
        case 'tictactoe':
            loadTicTacToeUI(container);
            break;
        case 'voting':
            loadVotingUI(container);
            break;
        case 'auction':
            loadAuctionUI(container);
            break;
        default:
            loadGenericUI(container, episodeType);
    }
}

function loadTicTacToeUI(container) {
    container.innerHTML = `
        <div class="tictactoe-game">
            <h2>Tic-Tac-Toe</h2>
            <div class="game-board">
                ${Array(9).fill().map((_, i) => 
                    `<div class="cell" data-index="${i}" onclick="makeMove(${i})"></div>`
                ).join('')}
            </div>
            <div class="game-status">Waiting for opponent...</div>
        </div>
    `;
}

function loadVotingUI(container) {
    container.innerHTML = `
        <div class="voting-app">
            <h2>Vote on This Topic</h2>
            <div class="voting-options" id="voting-options">
                <!-- Options will be loaded from Episode state -->
            </div>
            <div class="voting-results hidden" id="voting-results">
                <!-- Results will show after voting -->
            </div>
        </div>
    `;
}

function loadAuctionUI(container) {
    container.innerHTML = `
        <div class="auction-app">
            <h2>Live Auction</h2>
            <div class="current-bid">
                <span class="label">Current Bid:</span>
                <span class="amount" id="current-bid">0 KAS</span>
            </div>
            <div class="bid-form">
                <input type="number" id="bid-amount" placeholder="Your bid (KAS)" />
                <button onclick="placeBid()">Place Bid</button>
            </div>
            <div class="bid-history" id="bid-history">
                <!-- Bid history will appear here -->
            </div>
        </div>
    `;
}

function loadGenericUI(container, episodeType) {
    container.innerHTML = `
        <div class="generic-app">
            <h2>${episodeType} Application</h2>
            <p>This is a custom Episode type. Interact using the console or API.</p>
            <div class="episode-info">
                <p>Episode ID: ${window.episodeId}</p>
                <p>Type: ${episodeType}</p>
            </div>
        </div>
    `;
}

// UI update functions
function updateCountdown(element) {
    const expires = parseInt(element.dataset.expires);
    const now = Math.floor(Date.now() / 1000);
    const remaining = expires - now;
    
    if (remaining <= 0) {
        element.textContent = 'Expired';
        element.style.color = '#d9534f';
        return;
    }
    
    const hours = Math.floor(remaining / 3600);
    const minutes = Math.floor((remaining % 3600) / 60);
    
    if (hours > 24) {
        const days = Math.floor(hours / 24);
        element.textContent = `${days} day${days > 1 ? 's' : ''}`;
    } else if (hours > 0) {
        element.textContent = `${hours}h ${minutes}m`;
    } else {
        element.textContent = `${minutes} minutes`;
    }
    
    // Change color based on time remaining
    if (remaining < 3600) {
        element.style.color = '#f0ad4e'; // Warning - less than 1 hour
    } else if (remaining < 86400) {
        element.style.color = '#5bc0de'; // Info - less than 1 day
    }
}

function updateParticipantCount(count) {
    const element = document.querySelector('.participant-count');
    if (element) {
        element.textContent = `${count} participant${count !== 1 ? 's' : ''}`;
    }
}

function showExpirationWarning(remainingSeconds) {
    const minutes = Math.floor(remainingSeconds / 60);
    const warning = document.createElement('div');
    warning.className = 'expiration-warning';
    warning.innerHTML = `
        <strong>⚠️ Expiring Soon!</strong> 
        This Episode will expire in ${minutes} minutes.
    `;
    document.body.appendChild(warning);
    
    setTimeout(() => warning.remove(), 10000); // Remove after 10 seconds
}

// Action functions
function makeMove(index) {
    sendAction({
        type: 'GameMove',
        position: [index],
        player_id: getSessionId()
    });
}

function placeBid() {
    const amount = document.getElementById('bid-amount').value;
    if (!amount || amount <= 0) {
        alert('Please enter a valid bid amount');
        return;
    }
    
    sendAction({
        type: 'PlaceBid',
        amount: parseInt(amount),
        bidder_id: getSessionId()
    });
}

function sendAction(action) {
    if (window.episodeWs && window.episodeWs.readyState === WebSocket.OPEN) {
        // Get player number from URL params
        const urlParams = new URLSearchParams(window.location.search);
        const playerNumber = urlParams.get('player') ? parseInt(urlParams.get('player')) : null;
        
        if (playerNumber) {
            action.player = playerNumber;
        }
        
        window.episodeWs.send(JSON.stringify({
            type: 'Action',
            episode_id: window.episodeId,
            action: action
        }));
    } else {
        console.error('WebSocket not connected');
    }
}

function shareApp() {
    const url = window.location.href;
    
    // Try native share first (mobile)
    if (navigator.share) {
        navigator.share({
            title: `Join my ${window.episodeType || 'application'}`,
            text: `Check out this decentralized app on Kaspa!`,
            url: url
        }).catch(err => {
            console.log('Share failed:', err);
            copyToClipboard(url);
        });
    } else {
        // Fallback to clipboard copy
        copyToClipboard(url);
    }
}

function copyToClipboard(text) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(() => {
            showShareFeedback('Link copied to clipboard!');
        }).catch(err => {
            console.error('Clipboard write failed:', err);
            fallbackCopy(text);
        });
    } else {
        fallbackCopy(text);
    }
}

function fallbackCopy(text) {
    // Fallback for older browsers
    const textArea = document.createElement('textarea');
    textArea.value = text;
    textArea.style.position = 'fixed';
    textArea.style.opacity = '0';
    document.body.appendChild(textArea);
    textArea.select();
    try {
        document.execCommand('copy');
        showShareFeedback('Link copied to clipboard!');
    } catch (err) {
        showShareFeedback('Copy failed - please copy manually: ' + text);
    }
    document.body.removeChild(textArea);
}

function showShareFeedback(message) {
    // Show feedback near the share button
    const shareBtn = document.querySelector('.share-btn');
    if (shareBtn) {
        const originalText = shareBtn.textContent;
        shareBtn.textContent = 'Copied!';
        shareBtn.style.backgroundColor = '#28a745';
        setTimeout(() => {
            shareBtn.textContent = originalText;
            shareBtn.style.backgroundColor = '';
        }, 2000);
    } else {
        alert(message);
    }
}

// Utility functions
function getSessionId() {
    let sessionId = localStorage.getItem('kdapp_session_id');
    if (!sessionId) {
        sessionId = 'session_' + Math.random().toString(36).substr(2, 9);
        localStorage.setItem('kdapp_session_id', sessionId);
    }
    return sessionId;
}

function updateEpisodeState(state) {
    console.log('Episode state updated:', state);
    
    // Update based on Episode type
    const episodeType = window.episodeType;
    
    if (episodeType && episodeType.toLowerCase() === 'tictactoe') {
        updateTicTacToeBoard(state);
    }
    // Add other Episode types here
}

function updateTicTacToeBoard(state) {
    if (!state.board) return;
    
    // Update board cells
    const cells = document.querySelectorAll('.cell');
    state.board.forEach((row, rowIndex) => {
        row.forEach((cell, colIndex) => {
            const cellIndex = rowIndex * 3 + colIndex;
            const cellElement = cells[cellIndex];
            if (cellElement) {
                cellElement.textContent = cell || '';
                if (cell) {
                    cellElement.classList.add('filled');
                }
            }
        });
    });
    
    // Update game status
    const statusElement = document.querySelector('.game-status');
    if (statusElement) {
        if (state.status === 'winner' && state.winner) {
            statusElement.textContent = `Winner: ${state.winner}!`;
        } else if (state.status === 'draw') {
            statusElement.textContent = 'Game ended in a draw!';
        } else if (state.status === 'in_progress' && state.currentPlayer) {
            statusElement.textContent = `Current player: ${state.currentPlayer}`;
        } else {
            statusElement.textContent = 'Waiting for players...';
        }
    }
}

// Info section functions
function showInfo() {
    document.getElementById('info-section').classList.remove('hidden');
}

function hideInfo() {
    document.getElementById('info-section').classList.add('hidden');
}