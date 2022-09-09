const { createProxyMiddleware } = require('http-proxy-middleware');
module.exports = function(app) {
    app.use(
        '/jobs',
        createProxyMiddleware({
            target: 'http://127.0.0.1:12345',
            changeOrigin: true,
        })
    );
    app.use(
        '/contests',
        createProxyMiddleware({
            target: 'http://127.0.0.1:12345',
            changeOrigin: true,
        })
    );
    app.use(
        '/users',
        createProxyMiddleware({
            target: 'http://127.0.0.1:12345',
            changeOrigin: true,
        })
    );
}