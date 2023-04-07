"use strict";

const ENDPOINT = "https://tiktok-tts.weilnet.workers.dev";
const TEXT_BYTE_LIMIT = 300;

const textEncoder = new TextEncoder();

window.onload = () => {
  document.getElementById("charcount").textContent = `0/${TEXT_BYTE_LIMIT}`;

  fetch(`${ENDPOINT}/api/status`)
    .then((response) => response.json())
    .then((responseData) => {
      if (responseData.data?.available) {
        enableControls();
      } else {
        const message = `Service not available ${
          responseData.data.message && responseData.data.message.length > 1
            ? "<b>${responseData.data.message}</b>"
            : ""
        }, try again later or check the <a href='https://github.com/Weilbyte/tiktok-tts'>GitHub</a> repository for more info`;

        setError(message);
      }
    })
    .catch(() => {
      setError(
        "Error querying API status, try again later or check the <a href='https://github.com/Weilbyte/tiktok-tts'>GitHub</a> repository for more info"
      );
    });
};

const setError = (message) => {
  clearAudio();
  document.getElementById("error").style.display = "block";
  document.getElementById("errortext").innerHTML = message;
};

const clearError = () => {
  document.getElementById("error").style.display = "none";
  document.getElementById("errortext").innerHTML = "";
};

const setAudio = (base64, text) => {
  document.getElementById("success").style.display = "block";
  document.getElementById("audio").src = `data:audio/mpeg;base64,${base64}`;
  document.getElementById("generatedtext").innerHTML = `"${text}"`;
};

const clearAudio = () => {
  document.getElementById("success").style.display = "none";
  document.getElementById("audio").src = "";
  document.getElementById("generatedtext").innerHTML = "";
};

const disableControls = () => {
  document.getElementById("text").setAttribute("disabled", "");
  document.getElementById("voice").setAttribute("disabled", "");
  document.getElementById("submit").setAttribute("disabled", "");
};

const enableControls = () => {
  document.getElementById("text").removeAttribute("disabled");
  document.getElementById("voice").removeAttribute("disabled");
  document.getElementById("submit").removeAttribute("disabled");
};

const onTextareaInput = () => {
  const text = document.getElementById("text").value;
  const textEncoded = textEncoder.encode(text);
  const charCountElement = document.getElementById("charcount");

  charCountElement.textContent = `${
    textEncoded.length <= 999 ? textEncoded.length : 999
  }/${TEXT_BYTE_LIMIT}`;

  charCountElement.style.color =
    textEncoded.length > TEXT_BYTE_LIMIT ? "red" : "black";
};

const submitForm = async () => {
  clearError();
  clearAudio();
  disableControls();

  const voice = document.getElementById("voice").value;
  const text = document.getElementById("text").value;
  const textLength = new TextEncoder().encode(text).length;

  if (textLength === 0) text = "The fungus among us.";

  if (voice === "none") {
    setError("No voice has been selected");
    enableControls();
    return;
  }

  if (textLength > TEXT_BYTE_LIMIT) {
    setError(
      `Text must not be over ${TEXT_BYTE_LIMIT} UTF-8 characters (currently at ${textLength})`
    );
    enableControls();
    return;
  }

  try {
    const response = await fetch(`${ENDPOINT}/api/generation`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        text,
        voice,
      }),
    });

    const responseData = await response.json();

    if (!!!responseData.data) {
      setError(`<b>Generation failed</b><br/> ("${responseData.error}")`);
    } else {
      setAudio(responseData.data, text);
    }
  } catch (error) {
    setError("Error submitting form (printed to F12 console)");
  } finally {
    enableControls();
  }
};
