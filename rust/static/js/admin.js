// An admin panel function. When pressed, takes updates the relevant arbitrary value for the clicked button.
async function updateArbitraryValue(buttonElement) {
    const formElement = buttonElement.closest('form');

    const userInput = formElement.querySelector("input[type='text']").value;

    let fetchResult = await fetch("/admin/arbitrary_values", {
        method: 'PATCH',
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            arbitrary_value: formElement.dataset.arbitraryValue,
            set_to: userInput
        })
    });

    if (fetchResult.ok) {
        window.location.reload();
    }
    // TODO: Check for errors and float them to user.
}

// Function for the Art Archival Project. When pressed, updates the relevant art archive pin data.
async function updateArchivingProgressPin(buttonElement) {
    const formElement = buttonElement.closest('form');

    const discordLink = formElement.querySelector("input[type='text']").value;
    const messageDate = formElement.querySelector("input[type='date']").value;
    const pinUpdated = formElement.dataset.pinName;

    let body = JSON.stringify({
        updated_pin: pinUpdated,
        link: discordLink,
        date: messageDate
    });

    console.log(`SENDING ${body}`);

    let fetchResult = await fetch("/admin/art_archival_project", {
        method: 'PATCH',
        headers: {
            "Content-Type": "application/json"
        },
        body: body
    });

    if (fetchResult.ok) {
        window.location.reload();
    }

    // TODO: Check for errors and float them to user.
}