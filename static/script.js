const ENDPOINT = 'https://tiktok-tts.weilnet.workers.dev'

const TEXT_BYTE_LIMIT = 300
const textEncoder = new TextEncoder()

window.onload = () => {
    document.getElementById('charcount').textContent = `0`
    const req = new XMLHttpRequest()
    req.open('GET', `${ENDPOINT}/api/status`, false)
    req.send()

    let resp = JSON.parse(req.responseText)
    if (resp.data) {
        if (resp.data.available) {
            console.info(`${resp.data.meta.dc} (age ${resp.data.meta.age} minutes) is able to provide service`)
            enableControls()
        } else {
            console.error(`${resp.data.meta.dc} (age ${resp.data.meta.age} minutes) is unable to provide service`)
            setError(
                `Service not available${resp.data.message && resp.data.message.length > 1 ? ` (<b>"${resp.data.message}"</b>)` : ''}, try again later or check the <a href='https://github.com/Weilbyte/tiktok-tts'>GitHub</a> repository for more info`
                )
        }
    } else {
        setError('Error querying API status, try again later or check the <a href=\'https://github.com/Weilbyte/tiktok-tts\'>GitHub</a> repository for more info')
    }  
}

const setError = (message) => {
    clearAudio()
    document.getElementById('error').style.display = 'block'
    document.getElementById('errortext').innerHTML = message
}

const setLoading = () => {
    document.getElementById('loading').style.display = 'block'
}


const clearError = () => {
    document.getElementById('error').style.display = 'none'
    document.getElementById('errortext').innerHTML = 'There was an error.'
}

const clearLoading = () => {
    document.getElementById('loading').style.display = 'none'
}

const setAudio = (base64, text) => {
    document.getElementById('success').style.display = 'block'
    document.getElementById('audio').src = `data:audio/mpeg;base64,${base64}`
    document.getElementById('generatedtext').innerHTML = `"${text}"`
}

const clearAudio = () => {
    document.getElementById('success').style.display = 'none'
    document.getElementById('audio').src = ``
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
    const text = document.getElementById('text').value
    const textEncoded = textEncoder.encode(text)

    document.getElementById('charcount').textContent = `${textEncoded.length}`

    if (textEncoded.length > TEXT_BYTE_LIMIT) {
        document.getElementById('charcount').style.color = 'red'
    } else {
        document.getElementById('charcount').style.color = 'black'
    }
}

function splitIntoSubstrings(str, maxLength) {
    const substrings = [];
    const words = str.split(/([^\w])/);
    let currentSubstring = '';

    for (const word of words) {
      if (currentSubstring.length + word.length > maxLength) {
        substrings.push(currentSubstring);
        currentSubstring = '';
      }
      currentSubstring += word;
    }
    if (currentSubstring.length > 0) {
      substrings.push(currentSubstring);
    }
    return substrings;
  }

const submitForm = () => {
    clearError()
    clearAudio()

    disableControls()

    let text = document.getElementById('text').value
    const textLength = new TextEncoder().encode(text).length
    console.log(textLength)

    if (textLength === 0) text = 'The fungus among us.' 
    const voice = document.getElementById('voice').value

    if(voice == "none") {
        setError("No voice has been selected");
        enableControls()
        return
    }

    if (textLength > TEXT_BYTE_LIMIT) {
        setError(`Text must not be over ${TEXT_BYTE_LIMIT} UTF-8 characters (currently at ${textLength})`)
        enableControls()
        return
    }

    try {
        const req = new XMLHttpRequest()
        var text_array=splitIntoSubstrings(text,300);
        var decoded_audio;
        var final_audio;
        
        for (var i = 0; i < text_array.length; i++) {
            req.open('POST', `${ENDPOINT}/api/generation`, false)
            req.setRequestHeader('Content-Type', 'application/json')
            req.send(JSON.stringify({
                text: text_array[i],
                voice: voice
            }))
            setLoading()
            let resp = JSON.parse(req.responseText)
            if (resp.data === null) {
                setError(`<b>Generation failed</b><br/> ("${resp.error}")`)
            } else {
                var decoded=atob(resp.data)
                decoded_audio=decoded_audio+decoded
            }  
          }
          console.log(text_array)
          final_audio=btoa(decoded_audio)
          clearLoading()
          setAudio(final_audio, text)
    } catch {
        clearLoading()
        setError('Error submitting form (printed to F12 console)')
        console.log('^ Please take a screenshot of this and create an issue on the GitHub repository if one does not already exist :)')
        console.log('If the error code is 503, the service is currently unavailable. Please try again later.')
        console.log(`Voice: ${voice}`)
        console.log(`Text: ${text}`)
    }

    enableControls()
}
