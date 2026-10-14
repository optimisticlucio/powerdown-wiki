async function updateRitualData(targetUrl = window.location.pathname) {

    const invalidRitualists = document.querySelectorAll(
        '.character:has(textarea:valid):has(textarea:invalid)'
    );

    if (invalidRitualists.length) {
        updateErrorText("<b>Some ritualists have incomplete data!</b> Fill them out or delete them before submitting!");
        invalidRitualists[0].scrollIntoView({ behavior: 'smooth', block: 'center' });
        invalidRitualists[0].querySelector('textarea:invalid').focus();
        return;
    }

    const ritualistElements = document.querySelectorAll(".ritual-editor > .character");

    const ritualists = [...ritualistElements].flatMap((ritualistElement) => {
        let character_id = parseInt(ritualistElement.dataset.id);

        let power_name = ritualistElement.querySelector('textarea[name="powerName"]').value.trim();
        let power_description = ritualistElement.querySelector('textarea[name="powerDescription"]').value.trim();

        if (!(power_name && power_description)) {
            return [];
        }

        let data_to_send_back = {
            character_id,
            power_name,
            power_description,
        };

        return [data_to_send_back];
    });


    const messageToSend = {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "same-origin",
        body: JSON.stringify(ritualists)
    };

    console.log(`SENT: ${JSON.stringify(messageToSend)}`)

    updateErrorText(`Sending ritualist data...`);

    const categoryUpdateResponse = await fetch(targetUrl, messageToSend);

    if (categoryUpdateResponse.status >= 400 && categoryUpdateResponse.status < 600) {
        let errorText = await categoryUpdateResponse.text();
        updateErrorText(`<b>ERROR ${categoryUpdateResponse.status}, ${categoryUpdateResponse.statusText}:</b> ${errorText}`);
        return;
    }

    updateErrorText(`Upload successful!`);
    window.location.href = categoryUpdateResponse.url;
}
