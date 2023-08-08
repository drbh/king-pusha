self.addEventListener('push', function(event) {

    console.log('Received a push message', event);

    let message = event.data.text();

    var options = {
        body: message,
        icon: 'images/icon.png',
        badge: 'images/badge.png'
    };

    event.waitUntil(
        self.registration.showNotification('Push Notification', options)
    );
});
