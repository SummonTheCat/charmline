function getCookie(name) {
    const value = `; ${document.cookie}`;
    const parts = value.split(`; ${name}=`);
    if (parts.length === 2) return parts.pop().split(';').shift();
}

function setCookie(name, value, days) {
    const expires = new Date(Date.now() + days * 864e5).toUTCString();
    document.cookie = name + "=" + value + "; path=/; expires=" + expires;
}

function deleteCookie(name) {
    document.cookie = name + "=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT";
}

async function startSession() {
    const res = await fetch("/api/session/start", { method: "GET" });
    if (res.ok) {
        const data = await res.json();
        setCookie("session_id", data.session_id, 1);
        showSessionInfo(data);
    } else alert("Failed to start session.");
}

async function checkSessionLoop() {
    const sessionId = getCookie("session_id");
    if (!sessionId) {
        showStartButton();
        return;
    }

    const res = await fetch("/api/session/get", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ session_id: sessionId })
    });

    if (res.ok) {
        const data = await res.json();
        showSessionInfo(data);
    } else {
        deleteCookie("session_id");
        showStartButton();
    }
}

function showStartButton() {
    const panel = document.getElementById("session-panel");
    panel.style.display = "flex";
    panel.innerHTML = `
        <h2 style="font-family: 'Lora', serif;">No Active Session</h2>
        <button id="start-session-btn">Start Voice Chat</button>
    `;
    document.getElementById("start-session-btn").onclick = startSession;
}

function showSessionInfo(data) {
    document.getElementById("session-panel").style.display = "none";
    const chatUI = document.getElementById("chat-ui");
    chatUI.style.display = "flex";
    chatLog.textContent = "";
    if (data.chat) appendMessage("bot", data.chat);
    // Read the initial bot message aloud
    if (data.chat) speakText(data.chat);
}

const chatLog = document.getElementById("chat-log");
const micBtn = document.getElementById("mic-btn");
const voicePreview = document.getElementById("voice-preview");
const statusIndicator = document.getElementById("status-indicator");

function appendMessage(role, text) {
    const bubble = document.createElement("div");
    bubble.className = role === "user" ? "user-msg" : "bot-msg";
    bubble.textContent = text.trim();
    chatLog.appendChild(bubble);
    chatLog.scrollTop = chatLog.scrollHeight;
}

function showBotTyping() {
    const bubble = document.createElement("div");
    bubble.className = "bot-msg typing";
    bubble.textContent = "⋯";
    chatLog.appendChild(bubble);
    chatLog.scrollTop = chatLog.scrollHeight;
    return bubble;
}

async function sendInput(text) {
    const sessionId = getCookie("session_id");
    if (!sessionId) return console.warn("⚠️ No session active.");

    appendMessage("user", text);
    const loader = showBotTyping();
    voicePreview.textContent = "Bot is thinking...";

    const res = await fetch("/api/session/sendinput", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ session_id: sessionId, input: text })
    });

    loader.remove();
    if (res.ok) {
        const result = await res.json();

        if (result.session_ended) {
            const replyText = result.reply || "Session ended.";
            appendMessage("bot", replyText);
            speakText(replyText);
            handleSessionEnd(replyText);
        } else {
            const replyText = result.reply || result.error || "(no response)";
            appendMessage("bot", replyText);
            speakText(replyText);
            flashSentState();
        }
    } else {
        appendMessage("bot", "(Network error)");
    }

    voicePreview.textContent = "Waiting for voice...";
}


function handleSessionEnd(finalText) {
    deleteCookie("session_id");
    micBtn.classList.remove("listening", "sent");
    micBtn.style.opacity = "0.4";
    micBtn.style.pointerEvents = "none";
    statusIndicator.textContent = "Session Ended";
    voicePreview.textContent = "The conversation has ended.";

    // Speak the final bot message first
    speakText(finalText || "The conversation has ended.", () => {
        // Then speak the closing message
        speakText("The conversation has ended. Thank you for calling.", () => {
            // After both TTS messages, restart listening (if supported)
            if (recognition && !listening) {
                recognition.start();
                console.log("🎤 Listening resumed after TTS end.");
            }
            setTimeout(showStartButton, 4000);
        });
    });
}



window.debugSend = sendInput;

let recognition;
let listening = false;
let finalTranscript = "";
let interimTranscript = "";
let silenceTimer = null;
const silenceDelay = 3000; // wait a bit longer before sending
let audioCtx, analyser, micSource, dataArray, animationId;

async function startMicVisualizer() {
    try {
        if (!audioCtx) {
            audioCtx = new AudioContext();
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
            analyser = audioCtx.createAnalyser();
            micSource = audioCtx.createMediaStreamSource(stream);
            micSource.connect(analyser);
            analyser.fftSize = 256;
            dataArray = new Uint8Array(analyser.fftSize);
        }

        function animate() {
            analyser.getByteTimeDomainData(dataArray);
            let sum = 0;
            for (let i = 0; i < dataArray.length; i++) {
                const val = (dataArray[i] - 128) / 128;
                sum += val * val;
            }
            const rms = Math.sqrt(sum / dataArray.length);
            const scale = 1 + rms * 1.6;
            micBtn.style.transform = "scale(" + scale + ")";
            micBtn.style.boxShadow = "0 0 " + (rms * 60) + "px #4da3ff88";
            animationId = requestAnimationFrame(animate);
        }
        animate();
    } catch (err) {
        console.warn("🎤 Mic visualizer unavailable:", err);
    }
}

function stopMicVisualizer() {
    cancelAnimationFrame(animationId);
    micBtn.style.transform = "scale(1)";
    micBtn.style.boxShadow = "none";
}

if ("webkitSpeechRecognition" in window || "SpeechRecognition" in window) {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    recognition = new SpeechRecognition();
    recognition.continuous = true;
    recognition.interimResults = true;
    recognition.lang = "en-US";

    recognition.onstart = () => {
        listening = true;
        micBtn.classList.add("listening");
        statusIndicator.textContent = "Listening...";
        voicePreview.textContent = "Listening...";
        startMicVisualizer();
    };

    recognition.onend = () => {
        listening = false;
        micBtn.classList.remove("listening");
        statusIndicator.textContent = "Idle";
        if (!finalTranscript.trim()) voicePreview.textContent = "Waiting for voice...";
        stopMicVisualizer();
    };

    recognition.onresult = (event) => {
        interimTranscript = "";
        for (let i = event.resultIndex; i < event.results.length; ++i) {
            const result = event.results[i];
            const text = result[0].transcript.trim();

            if (result.isFinal) {
                finalTranscript += " " + text;
            } else {
                interimTranscript += " " + text;
            }
        }

        const display = (finalTranscript + " " + interimTranscript).trim();
        voicePreview.textContent = display || "Listening...";

        if (silenceTimer) clearTimeout(silenceTimer);

        silenceTimer = setTimeout(async () => {
            const textToSend = finalTranscript.trim();
            if (!textToSend) return;

            voicePreview.textContent = "Bot is thinking...";
            finalTranscript = "";
            interimTranscript = "";
            await sendInput(textToSend);
        }, silenceDelay);
    };
}

function toggleMic() {
    if (!recognition) return;
    if (!listening) recognition.start();
    else recognition.stop();
}

function flashSentState() {
    micBtn.classList.remove("listening");
    micBtn.classList.add("sent");
    statusIndicator.textContent = "Message sent";
    voicePreview.textContent = "Message sent";
    setTimeout(() => {
        micBtn.classList.remove("sent");
        if (listening) micBtn.classList.add("listening");
        statusIndicator.textContent = listening ? "Listening..." : "Idle";
        if (!listening) voicePreview.textContent = "Waiting for voice...";
    }, 800);
}

// --- Client-side Text-to-Speech (TTS) ---
let ttsEnabled = true;

function toggleTTS() {
    ttsEnabled = !ttsEnabled;
    if (!ttsEnabled) {
        window.speechSynthesis.cancel();
        console.log("🔇 TTS muted");
    } else {
        console.log("🔊 TTS enabled");
    }
}

function speakText(text, onEndCallback) {
    if (!ttsEnabled || !window.speechSynthesis) return;

    // Stop any ongoing speech
    window.speechSynthesis.cancel();

    const utterance = new SpeechSynthesisUtterance(text);
    utterance.lang = "en-ZA"; // South African English if available
    utterance.rate = 1.0;
    utterance.pitch = 1.0;
    utterance.volume = 1.0;

    // Pick the most natural voice
    const voices = window.speechSynthesis.getVoices();
    const preferred =
        voices.find(v => v.lang.startsWith("en-ZA")) ||
        voices.find(v => v.lang.startsWith("en-GB")) ||
        voices.find(v => v.lang.startsWith("en-US"));
    if (preferred) utterance.voice = preferred;

    utterance.onstart = () => {
        statusIndicator.textContent = "Speaking...";
        micBtn.style.opacity = "0.5";
    };

    utterance.onend = () => {
        statusIndicator.textContent = "Idle";
        micBtn.style.opacity = "1";

        // Automatically resume listening when TTS ends
        if (recognition && !listening) {
            try {
                recognition.start();
                console.log("🎤 Listening resumed after TTS.");
            } catch (err) {
                console.warn("SpeechRecognition restart failed:", err);
            }
        }

        if (typeof onEndCallback === "function") onEndCallback();
    };

    window.speechSynthesis.speak(utterance);
}


// preload voices in Chrome
if (window.speechSynthesis) {
    window.speechSynthesis.onvoiceschanged = () => { };
}



window.addEventListener("DOMContentLoaded", checkSessionLoop);
