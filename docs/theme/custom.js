// Custom JavaScript for vLLM Client documentation
// Add custom functionality here

(function() {
    'use strict';

    // Add language switcher to menu bar
    document.addEventListener('DOMContentLoaded', function() {
        console.log('vLLM Client documentation loaded');

        // Create language selector
        var langSelector = document.createElement('div');
        langSelector.className = 'language-selector';

        // Determine current language based on URL
        var isZh = window.location.pathname.includes('/zh/');

        // Create links
        var enLink = document.createElement('a');
        enLink.href = '/vllm-client/';
        enLink.textContent = 'English';
        if (!isZh) enLink.className = 'active';

        var zhLink = document.createElement('a');
        zhLink.href = '/vllm-client/zh/';
        zhLink.textContent = '中文';
        if (isZh) zhLink.className = 'active';

        langSelector.appendChild(enLink);
        langSelector.appendChild(zhLink);

        // Find menu bar and insert language selector after left-buttons
        var menuBar = document.querySelector('#menu-bar .left-buttons');
        if (menuBar) {
            menuBar.parentNode.insertBefore(langSelector, menuBar.nextSibling);
        }
    });
})();
