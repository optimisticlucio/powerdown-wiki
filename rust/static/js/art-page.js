document.addEventListener('DOMContentLoaded', () => {
    const radios = document.querySelectorAll('input[name="art"]');
    const artItems = document.querySelectorAll('.art-display .art-item');
    const labels = document.querySelectorAll('.controls label');
    const videos = document.querySelectorAll('video');

    radios.forEach((radio, index) => {
        radio.addEventListener('change', () => {
            // Pause and reset all videos
            videos.forEach(video => {
                video.pause();
                video.currentTime = 0;
            });

            // Remove active class from all items and labels
            artItems.forEach(item => item.classList.remove('active'));
            labels.forEach(label => label.classList.remove('active'));

            // Add active class to selected item and label
            artItems[index].classList.add('active');
            labels[index].classList.add('active');
        });
    });

    // Initialize first item as active
    if (artItems[0]) artItems[0].classList.add('active');
    if (labels[0]) labels[0].classList.add('active');
});

function postComment() {
    const commentText = document.getElementById("commentArea").value;

    fetch(`${window.location.pathname}/comment`, {
        method: 'POST',
        headers: {
            'Content-Type': 'text/plain'
        },
        body: commentText
    })
        .then(response => {
            if (response.ok) {
                window.location.reload();
            } else {
                console.error('Error:', response.statusText);
            }
        })
        .catch(error => {
            console.error('Error:', error);
        });
}