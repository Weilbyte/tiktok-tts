const ENDPOINT = 'https://tiktok-tts.weilnet.workers.dev'

let SESSION_ID = null

window.onload = () => {
    const req = new XMLHttpRequest()
    req.open('GET', `${ENDPOINT}/session`, false)
    req.send()

    let resp = JSON.parse(req.responseText)
    if (resp.available) {
        SESSION_ID = resp.session_id
        console.log(`Got SID ${SESSION_ID} from account pool ${resp.account_pool}`)
        enableControls()
    } else {
        setError(`Service not available, try again later or check in on GitHub for updates`)
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

const submitForm = () => {
    clearError()
    clearAudio()

    if (SESSION_ID === null) {
        setError('Session ID missing, please reload page')
        return
    }

    disableControls()

    let text = document.getElementById('text').value
    if (text.length === 0) text = 'The fungus among us.' 
    const voice = document.getElementById('voice').value

    try {
        const req = new XMLHttpRequest()
        req.open('POST', `${ENDPOINT}/tts`, false)
        req.send(JSON.stringify({
            session_id: SESSION_ID,
            text: text,
            voice: voice
        }))

        let resp = JSON.parse(req.responseText)
        if (resp.data === null) {
            setError(`Generation failed. Make sure your text is not too long. (Generation ID ${resp.ga})`)
        } else {
            setAudio(resp.data, text)
        }  
    } catch {
        console.log(`SID: ${SESSION_ID}\nText: ${text}\nVoice: ${voice}`)
        setError('Error submitting form (printed to console). Please raise an issue on the GitHub repository.')
        console.log('^ Please take a screenshot of this and create an issue on the GitHub repository :)')
    }

    enableControls()
}
