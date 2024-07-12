const TEXT_BYTE_LIMIT = 300
const textEncoder = new TextEncoder()

window.onload = () => {
    document.getElementById('charcount').textContent = `0/${TEXT_BYTE_LIMIT}`
    const req = new XMLHttpRequest()
    req.open('GET', `/api/status`, false)
    req.send()

    if (req.status === 200) {
        enableControls()
    } else {
        setError(
            `Service not available: ${req.responseText}`
        )
    }
}

const setError = (message) => {
    clearAudio()
    document.getElementById('error').style.display = 'block'
    document.getElementById('errortext').innerHTML = message
}

const clearError = () => {
    document.getElementById('error').style.display = 'none'
    document.getElementById('errortext').innerHTML = 'There was an error.'
}

const setAudio = (base64, text) => {
    document.getElementById('success').style.display = 'block'
    document.getElementById('audio').src = `data:audio/mpeg;base64,${base64}`
    document.getElementById('audio-link').href = `data:audio/mpeg;base64,${base64}`

    let now = new Date();
    let day = now.getDate().toString().padStart(2, '0');
    let hours = now.getHours().toString().padStart(2, '0');
    let minutes = now.getMinutes().toString().padStart(2, '0');
    let seconds = now.getSeconds().toString().padStart(2, '0');
    let safeText = text.replace(/[<>:"\/\\|?*]+/g, '_').replaceAll(' ', '_').substring(0, 16);
    document.getElementById('audio-link').download = `${day}${hours}${minutes}${seconds}_${safeText}.mp3`
    document.getElementById('audio').download = `${day}${hours}${minutes}${seconds}_${safeText}.mp3`
    document.getElementById('generatedtext').innerHTML = `"${text}"`
}

const clearAudio = () => {
    document.getElementById('success').style.display = 'none'
    document.getElementById('audio').src = ''
    document.getElementById('audio-link').href = ''
    document.getElementById('audio-link').download = null
    document.getElementById('generatedtext').innerHTML = ''
}

const disableControls = () => {
    document.getElementById('text').setAttribute('disabled', '')
    document.getElementById('voice').setAttribute('disabled', '')
    document.getElementById('submit').setAttribute('disabled', '')
}

const enableControls = () => {
    document.getElementById('text').removeAttribute('disabled')
    document.getElementById('voice').removeAttribute('disabled')
    document.getElementById('submit').removeAttribute('disabled')
}

const onTextareaInput = () => {
    const textEncoded = textEncoder.encode(document.getElementById('text').value)

    if (textEncoded.length > TEXT_BYTE_LIMIT) {
        document.getElementById('charcount').textContent = `${textEncoded.length} ⚠️`
        document.getElementById('charcount').title = `Audio will be chunked, stay below ${TEXT_BYTE_LIMIT} characters for best quality`
    } else {
        document.getElementById('charcount').textContent = textEncoded.length.toString()
        document.getElementById('charcount').title = null
    }
}

const submitForm = () => {
    clearError()
    clearAudio()

    disableControls()

    let text = document.getElementById('text').value
    const textLength = new TextEncoder().encode(text).length

    if (textLength === 0) text = 'The fungus among us.'
    const voice = document.getElementById('voice').value

    if (voice == "none") {
        setError("No voice has been selected");
        enableControls()
        return
    }

    try {
        const req = new XMLHttpRequest()
        req.open('POST', `/api/generate`, false)
        req.setRequestHeader('Content-Type', 'application/json')
        req.send(JSON.stringify({
            text: text,
            voice: voice,
            base64: true
        }))

        if (req.status === 200) {
            setAudio(req.responseText, text)
        } else {
            setError(`<b>Generation failed</b><br/> ("${req.responseText}")`)
        }
    } catch (e) {
        setError('Error submitting form (printed to F12 console)')
        console.error(e)
        console.log('If the error code is 503, the service is currently unavailable. Please try again later.')
        console.log(`Voice: ${voice}`)
        console.log(`Text: ${text}`)
    }

    enableControls()
}

const displayAPIHelp = () => {
    document.getElementById('api-access-button').textContent = 'Check developer console'

    console.group('API Documentation');

    console.log('%cGET /api/status', 'color: blue; font-weight: bold;');
    console.log('Description: Fetch the status of the API.');
    console.log(`Endpoint: %cGET ${window.location.href}api/status`, 'font-style: italic;');
    console.log('Responses:');
    console.log('   %c200: Service is available.', 'color: green;');
    console.log('   %cOther: Service is not available; response text is error message.', 'color: red;');

    console.log('');

    console.log('%cPOST /api/generate', 'color: blue; font-weight: bold;');
    console.log('Description: Generate TTS based on provided text and voice settings.');
    console.log(`Endpoint: %cPOST ${window.location.href}api/generate`, 'font-style: italic;');
    console.log('Request Body (JSON):');
    console.log('   %ctext: The text to generate TTS for (less than 300 bytes for best quality; longer texts will be chunked and quality may degrade).', 'color: black;');
    console.log('   %cvoice: The TTS voice to use (e.g., `en_us_001`).', 'color: black;');
    console.log('Responses:');
    console.log('   %c200: Returns an MP3 audio in binary format with `application/octet-stream` content type.', 'color: green;');
    console.log('   %c400: User error encountered during TTS generation; response text is error message.', 'color: red;');
    console.log('   %c429: You have hit a request size limit! Try again after timestamp in `Retry-After` header.', 'color: red;');
    console.log('   %c500: Error encountered during TTS generation; response text is error message.', 'color: red;');
    console.log('   %c503: Service not available; response text is error message.', 'color: red;');


    console.groupEnd();
}
