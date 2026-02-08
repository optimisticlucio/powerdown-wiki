// Sends the changes that the client did to the server.
async function sendUserModifications() {
    const targetUrl = "."; 
    let changesDone = {};

    const newDisplayName = document.getElementById("userDisplayName").value.trim();
    if (newDisplayName) {
        changesDone.display_name = newDisplayName;
    }

    const newProfilePic = document.getElementById("userProfilePicture").files[0];
    if (newProfilePic) {
        // Get the S3 presigned to put the picture in.
        const messageToSend = {
            method: "PATCH",
            headers: {
            "Content-Type": "application/json"
            },
            credentials: "same-origin",
            body: JSON.stringify({
            step: "1",
            file_amount: 1
            })
        };

        const s3UrlsRequestResponse = await fetch(targetUrl, messageToSend);
        // I'm just going to assume the response was valid. TODO: Handle error and show to user.

        let s3Url = await s3UrlsRequestResponse.json();
        s3Url = s3Url.presigned_urls.pop();

        await fetch(s3Url, {
            method: 'PUT',
            body: newProfilePic,
            headers: {
            'Content-Type': newProfilePic.type
            }
        })

        changesDone.pfp_temp_key = s3Url;
    }

    const newCreatorName = document.getElementById("userCreatorName").value.trim();
    if (newCreatorName) {
        changesDone.creator_name = newCreatorName;
    }

    const newUserType = document.getElementById("userType")?.value;
    if (newUserType) {
        changesDone.user_type = newUserType;
    }

    await fetch(targetUrl, {
        method: 'PATCH',
        headers: {
        "Content-Type": "application/json"
        },
        body: JSON.stringify({
        step: "2",
        ...changesDone
        })
    });

    // TODO: Check for errors and float them to user.

    window.location.reload();
}