# King Pusha

### 🍬 Whadda I do?

I run a small web app and server that sends push notifications to your device. If you have an iOS device you can save the app to your home screen and receive push notifications from the server similar to native apps! 🤯

tldr; Save [webpush.drbh.xyz](https://webpush.drbh.xyz/) to your home screen, click subscribe and send. You'll receive a push notification on your device. 🎉

### 🔬 Deets

1. Web Push is a thing [learn more](https://web.dev/push-notifications-web-push-protocol/)
2. It's really cool and it makes web apps much more like native apps
3. This year Apple announced [Web Push for Web Apps on iOS and iPadOS](https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/)
4. This means I no longer need to have a native app to send push notifications to my iphone! wooooo 🎉
5. 🗒️ note** Web Push are only supported if you save the website to your home screen.
6. This is a small app to play with these things.
7. This app aims be be informative and a small piece of art.
8. The server code all lives in [src/main.rs](src/main.rs)
9.  Its built on actix (server), web-push (vapid key and etc..) and paperclip (built in REST docs)
10. The frontend portion is just HTML and native JS in [web](web). No frameworks. No libraries.
11. We do however need a [service worker](web/service-worker.js) to handle the push notifications. This is required because the app needs to be able to run in the background (listening for messages).
12. 🗒️ note** you'll need to set your own VAPID pem key in the `.env` file. and corresponding public key [in the web app](web/index.html#L75)
13. run the server with `cargo run`
14. open the app in your browser at [http://localhost:8081](http://localhost:8081)
15. you can access the OpenAPI JSON at [http://localhost:8081/api/spec/v2](http://localhost:8081/api/spec/v2)