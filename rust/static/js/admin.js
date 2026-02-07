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
        arbitrary_value: formElement.arbitraryValue,
        set_to: userInput
        })
    });

    if (fetchResult.ok) {
        window.location.reload();
    }
    // TODO: Check for errors and float them to user.
}