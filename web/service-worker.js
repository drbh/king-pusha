self.addEventListener('push', function(event) {

    console.log('Received a push message', event);

    var options = {
        body: 'This is a push notification',
        icon: 'images/icon.png',
        badge: 'images/badge.png'
    };

    event.waitUntil(
        self.registration.showNotification('Push Notification', options)
    );
});
