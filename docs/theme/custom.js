// Custom JavaScript for vLLM Client documentation

(function() {
    'use strict';

    // ========================================
    // Language Switcher for Documentation Pages
    // ========================================

    document.addEventListener('DOMContentLoaded', function() {
        // Skip if we're on the landing page (has particles canvas)
        if (document.getElementById('particles-canvas')) {
            return;
        }

        console.log('vLLM Client documentation loaded');

        // Create language selector
        var langSelector = document.createElement('div');
        langSelector.className = 'language-selector';

        // Determine current language based on URL
        var isZh = window.location.pathname.includes('/zh/');
        var currentPage = window.location.pathname;

        // Create links - always point to the landing page
        var enLink = document.createElement('a');
        enLink.href = '/vllm-client/';
        enLink.textContent = 'EN';
        if (!isZh) enLink.className = 'active';

        var zhLink = document.createElement('a');
        zhLink.href = '/vllm-client/zh/';
        zhLink.textContent = '中文';
        if (isZh) zhLink.className = 'active';

        langSelector.appendChild(enLink);
        langSelector.appendChild(zhLink);

        // Find menu bar and insert language selector
        var menuBar = document.querySelector('#menu-bar .left-buttons');
        if (menuBar) {
            // Create a container for our lang selector
            var container = document.createElement('div');
            container.className = 'lang-selector-container';
            container.appendChild(langSelector);

            // Insert after the theme toggle button area
            var themeToggle = document.querySelector('#theme-toggle');
            if (themeToggle && themeToggle.parentNode) {
                themeToggle.parentNode.insertBefore(container, themeToggle.nextSibling);
            } else if (menuBar.nextSibling) {
                menuBar.parentNode.insertBefore(container, menuBar.nextSibling);
            } else {
                menuBar.parentNode.appendChild(container);
            }
        }
    });

    // ========================================
    // Particle Background Animation (Landing Page Only)
    // ========================================

    function initParticles() {
        var canvas = document.getElementById('particles-canvas');
        if (!canvas) return;

        var ctx = canvas.getContext('2d');
        var particles = [];
        var particleCount = 80;
        var connectionDistance = 150;
        var mouseDistance = 200;

        // Set canvas size
        function resizeCanvas() {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
        }
        resizeCanvas();
        window.addEventListener('resize', resizeCanvas);

        // Mouse position
        var mouse = {
            x: window.innerWidth / 2,
            y: window.innerHeight / 2
        };

        document.addEventListener('mousemove', function(e) {
            mouse.x = e.clientX;
            mouse.y = e.clientY;
        });

        // Particle class
        function Particle(x, y) {
            this.x = x || Math.random() * canvas.width;
            this.y = y || Math.random() * canvas.height;
            this.vx = (Math.random() - 0.5) * 0.5;
            this.vy = (Math.random() - 0.5) * 0.5;
            this.radius = Math.random() * 2 + 1;
            this.color = 'rgba(72, 219, 251, ' + (Math.random() * 0.5 + 0.2) + ')';
        }

        Particle.prototype.update = function() {
            this.x += this.vx;
            this.y += this.vy;

            // Wrap around edges
            if (this.x < 0) this.x = canvas.width;
            if (this.x > canvas.width) this.x = 0;
            if (this.y < 0) this.y = canvas.height;
            if (this.y > canvas.height) this.y = 0;
        };

        Particle.prototype.draw = function() {
            ctx.beginPath();
            ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
            ctx.fillStyle = this.color;
            ctx.fill();
        };

        // Create particles
        for (var i = 0; i < particleCount; i++) {
            particles.push(new Particle());
        }

        // Draw connections
        function drawConnections() {
            for (var i = 0; i < particles.length; i++) {
                for (var j = i + 1; j < particles.length; j++) {
                    var dx = particles[i].x - particles[j].x;
                    var dy = particles[i].y - particles[j].y;
                    var dist = Math.sqrt(dx * dx + dy * dy);

                    if (dist < connectionDistance) {
                        var opacity = 1 - (dist / connectionDistance);
                        ctx.beginPath();
                        ctx.strokeStyle = 'rgba(72, 219, 251, ' + (opacity * 0.3) + ')';
                        ctx.lineWidth = 1;
                        ctx.moveTo(particles[i].x, particles[i].y);
                        ctx.lineTo(particles[j].x, particles[j].y);
                        ctx.stroke();
                    }
                }
            }

            // Connect particles to mouse
            for (var i = 0; i < particles.length; i++) {
                var dx = particles[i].x - mouse.x;
                var dy = particles[i].y - mouse.y;
                var dist = Math.sqrt(dx * dx + dy * dy);

                if (dist < mouseDistance) {
                    var opacity = 1 - (dist / mouseDistance);
                    ctx.beginPath();
                    ctx.strokeStyle = 'rgba(255, 107, 107, ' + (opacity * 0.5) + ')';
                    ctx.lineWidth = 1.5;
                    ctx.moveTo(particles[i].x, particles[i].y);
                    ctx.lineTo(mouse.x, mouse.y);
                    ctx.stroke();
                }
            }
        }

        // Animation loop
        function animate() {
            ctx.clearRect(0, 0, canvas.width, canvas.height);

            // Update and draw particles
            for (var i = 0; i < particles.length; i++) {
                particles[i].update();
                particles[i].draw();
            }

            drawConnections();
            requestAnimationFrame(animate);
        }

        animate();
    }

    // Initialize particles on load
    initParticles();

})();
