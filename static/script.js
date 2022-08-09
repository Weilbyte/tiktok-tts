//const CORS_PROXY = 'https://warp-co.rs/'
const CORS_PROXY = 'https://api.allorigins.win/raw?url=' // Temporary until warp-co.rs certificate is renewed
const ENDPOINT = 'https://api16-normal-useast5.us.tiktokv.com/media/api/text/speech/invoke/'

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

const submitForm = () => {
    clearError()
    clearAudio()

    document.getElementById('text').setAttribute('disabled', '')
    document.getElementById('voice').setAttribute('disabled', '')
    document.getElementById('submit').setAttribute('disabled', '')

    let text = document.getElementById('text').value
    if (text.length === 0) text = 'The fungus among us.' 
    const voice = document.getElementById('voice').value

    try {
        const req = new XMLHttpRequest()
        req.open('POST', `${CORS_PROXY}${ENDPOINT}?text_speaker=${voice}&req_text=${encodeURIComponent(text)}`, false)
        req.send()

        let resp = JSON.parse(req.responseText)
        if (resp.status_code != 0) {
            setError(`Server returned error (message: "${resp.message}").`)
        } else {
            setAudio(resp.data.v_str, text)
        }  
    } catch {
        setError('Error submitting form (printed to console). Please raise an issue on the GitHub repository.')
        console.log('^ Please take a screenshot of this and create an issue on the GitHub repository :)')
    }

    document.getElementById('text').removeAttribute('disabled')
    document.getElementById('voice').removeAttribute('disabled')
    document.getElementById('submit').removeAttribute('disabled')
}
