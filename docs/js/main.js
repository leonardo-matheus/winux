/**
 * Winux OS - Landing Page JavaScript
 * Modern interactions and animations
 */

(function() {
    'use strict';

    // ============================================
    // DOM Elements
    // ============================================
    const navbar = document.getElementById('navbar');
    const navToggle = document.getElementById('nav-toggle');
    const navMenu = document.getElementById('nav-menu');
    const navLinks = document.querySelectorAll('.nav-link');
    const faqItems = document.querySelectorAll('.faq-item');
    const carouselTrack = document.getElementById('carousel-track');
    const carouselPrev = document.getElementById('carousel-prev');
    const carouselNext = document.getElementById('carousel-next');
    const carouselDots = document.querySelectorAll('.dot');
    const animatedElements = document.querySelectorAll('[data-animate]');

    // ============================================
    // Navbar Scroll Effect
    // ============================================
    let lastScrollY = window.scrollY;
    let ticking = false;

    function updateNavbar() {
        const scrollY = window.scrollY;

        if (scrollY > 50) {
            navbar.classList.add('scrolled');
        } else {
            navbar.classList.remove('scrolled');
        }

        lastScrollY = scrollY;
        ticking = false;
    }

    window.addEventListener('scroll', function() {
        if (!ticking) {
            window.requestAnimationFrame(updateNavbar);
            ticking = true;
        }
    });

    // ============================================
    // Mobile Navigation Toggle
    // ============================================
    if (navToggle && navMenu) {
        navToggle.addEventListener('click', function() {
            navToggle.classList.toggle('active');
            navMenu.classList.toggle('active');
            document.body.style.overflow = navMenu.classList.contains('active') ? 'hidden' : '';
        });

        // Close menu on link click
        navLinks.forEach(function(link) {
            link.addEventListener('click', function() {
                navToggle.classList.remove('active');
                navMenu.classList.remove('active');
                document.body.style.overflow = '';
            });
        });

        // Close menu on outside click
        document.addEventListener('click', function(e) {
            if (!navMenu.contains(e.target) && !navToggle.contains(e.target)) {
                navToggle.classList.remove('active');
                navMenu.classList.remove('active');
                document.body.style.overflow = '';
            }
        });
    }

    // ============================================
    // Smooth Scroll for Anchor Links
    // ============================================
    document.querySelectorAll('a[href^="#"]').forEach(function(anchor) {
        anchor.addEventListener('click', function(e) {
            const targetId = this.getAttribute('href');
            if (targetId === '#') return;

            const targetElement = document.querySelector(targetId);
            if (targetElement) {
                e.preventDefault();
                const headerOffset = 80;
                const elementPosition = targetElement.getBoundingClientRect().top;
                const offsetPosition = elementPosition + window.pageYOffset - headerOffset;

                window.scrollTo({
                    top: offsetPosition,
                    behavior: 'smooth'
                });
            }
        });
    });

    // ============================================
    // FAQ Accordion
    // ============================================
    faqItems.forEach(function(item) {
        const question = item.querySelector('.faq-question');

        question.addEventListener('click', function() {
            const isActive = item.classList.contains('active');

            // Close all other items
            faqItems.forEach(function(otherItem) {
                otherItem.classList.remove('active');
                otherItem.querySelector('.faq-question').setAttribute('aria-expanded', 'false');
            });

            // Toggle current item
            if (!isActive) {
                item.classList.add('active');
                question.setAttribute('aria-expanded', 'true');
            }
        });

        // Keyboard accessibility
        question.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                question.click();
            }
        });
    });

    // ============================================
    // Screenshots Carousel
    // ============================================
    let currentSlide = 0;
    const slides = document.querySelectorAll('.screenshot-item');
    const totalSlides = slides.length;

    function updateCarousel() {
        if (!carouselTrack) return;

        // Update track position
        carouselTrack.style.transform = `translateX(-${currentSlide * 100}%)`;

        // Update active slide
        slides.forEach(function(slide, index) {
            slide.classList.toggle('active', index === currentSlide);
        });

        // Update dots
        carouselDots.forEach(function(dot, index) {
            dot.classList.toggle('active', index === currentSlide);
        });
    }

    function nextSlide() {
        currentSlide = (currentSlide + 1) % totalSlides;
        updateCarousel();
    }

    function prevSlide() {
        currentSlide = (currentSlide - 1 + totalSlides) % totalSlides;
        updateCarousel();
    }

    if (carouselNext) {
        carouselNext.addEventListener('click', nextSlide);
    }

    if (carouselPrev) {
        carouselPrev.addEventListener('click', prevSlide);
    }

    // Dot navigation
    carouselDots.forEach(function(dot, index) {
        dot.addEventListener('click', function() {
            currentSlide = index;
            updateCarousel();
        });
    });

    // Auto-advance carousel
    let carouselInterval = setInterval(nextSlide, 5000);

    // Pause on hover
    if (carouselTrack) {
        carouselTrack.parentElement.addEventListener('mouseenter', function() {
            clearInterval(carouselInterval);
        });

        carouselTrack.parentElement.addEventListener('mouseleave', function() {
            carouselInterval = setInterval(nextSlide, 5000);
        });
    }

    // Touch/swipe support for carousel
    let touchStartX = 0;
    let touchEndX = 0;

    if (carouselTrack) {
        carouselTrack.addEventListener('touchstart', function(e) {
            touchStartX = e.changedTouches[0].screenX;
        }, { passive: true });

        carouselTrack.addEventListener('touchend', function(e) {
            touchEndX = e.changedTouches[0].screenX;
            handleSwipe();
        }, { passive: true });
    }

    function handleSwipe() {
        const swipeThreshold = 50;
        const diff = touchStartX - touchEndX;

        if (Math.abs(diff) > swipeThreshold) {
            if (diff > 0) {
                nextSlide();
            } else {
                prevSlide();
            }
        }
    }

    // ============================================
    // Intersection Observer for Animations
    // ============================================
    const observerOptions = {
        root: null,
        rootMargin: '0px',
        threshold: 0.1
    };

    const observer = new IntersectionObserver(function(entries) {
        entries.forEach(function(entry) {
            if (entry.isIntersecting) {
                entry.target.classList.add('animate');
                observer.unobserve(entry.target);
            }
        });
    }, observerOptions);

    animatedElements.forEach(function(element) {
        observer.observe(element);
    });

    // ============================================
    // Staggered Animation for Grid Items
    // ============================================
    function animateGridItems(selector, delay) {
        const items = document.querySelectorAll(selector);
        items.forEach(function(item, index) {
            item.style.transitionDelay = `${index * delay}ms`;
        });
    }

    animateGridItems('.feature-card', 100);
    animateGridItems('.app-card', 50);
    animateGridItems('.faq-item', 100);

    // ============================================
    // Parallax Effect for Hero
    // ============================================
    const heroGradient = document.querySelector('.hero-gradient');

    if (heroGradient) {
        window.addEventListener('scroll', function() {
            const scrollY = window.scrollY;
            if (scrollY < window.innerHeight) {
                heroGradient.style.transform = `translateX(-50%) translateY(${scrollY * 0.3}px)`;
            }
        }, { passive: true });
    }

    // ============================================
    // Active Navigation Link Highlighting
    // ============================================
    const sections = document.querySelectorAll('section[id]');

    function highlightNavLink() {
        const scrollY = window.scrollY;

        sections.forEach(function(section) {
            const sectionTop = section.offsetTop - 100;
            const sectionHeight = section.offsetHeight;
            const sectionId = section.getAttribute('id');

            if (scrollY >= sectionTop && scrollY < sectionTop + sectionHeight) {
                navLinks.forEach(function(link) {
                    link.classList.remove('active');
                    if (link.getAttribute('href') === `#${sectionId}`) {
                        link.classList.add('active');
                    }
                });
            }
        });
    }

    window.addEventListener('scroll', highlightNavLink, { passive: true });

    // ============================================
    // Terminal Typing Animation
    // ============================================
    const typingElement = document.querySelector('.typing');

    if (typingElement) {
        const text = typingElement.textContent;
        typingElement.textContent = '';
        let charIndex = 0;

        function typeChar() {
            if (charIndex < text.length) {
                typingElement.textContent += text.charAt(charIndex);
                charIndex++;
                setTimeout(typeChar, 50);
            } else {
                typingElement.classList.remove('typing');
                typingElement.style.borderRight = 'none';
            }
        }

        // Start typing after a delay
        setTimeout(typeChar, 1000);
    }

    // ============================================
    // Download Button Analytics (placeholder)
    // ============================================
    const downloadButtons = document.querySelectorAll('.download-btn');

    downloadButtons.forEach(function(button) {
        button.addEventListener('click', function() {
            // Analytics tracking placeholder
            console.log('Download initiated');

            // You can add actual analytics here
            // gtag('event', 'download', { ... });
        });
    });

    // ============================================
    // Copy Code to Clipboard (for terminal)
    // ============================================
    const mockupContent = document.querySelector('.mockup-content');

    if (mockupContent) {
        mockupContent.addEventListener('dblclick', function() {
            const command = 'winux install firefox vscode docker';
            navigator.clipboard.writeText(command).then(function() {
                // Show feedback
                const feedback = document.createElement('div');
                feedback.textContent = 'Command copied!';
                feedback.style.cssText = `
                    position: absolute;
                    top: 50%;
                    left: 50%;
                    transform: translate(-50%, -50%);
                    background: var(--color-primary);
                    color: white;
                    padding: 8px 16px;
                    border-radius: 8px;
                    font-size: 14px;
                    z-index: 10;
                    animation: fadeIn 0.3s ease;
                `;
                mockupContent.style.position = 'relative';
                mockupContent.appendChild(feedback);

                setTimeout(function() {
                    feedback.remove();
                }, 2000);
            });
        });
    }

    // ============================================
    // Performance: Defer non-critical animations
    // ============================================
    if ('requestIdleCallback' in window) {
        requestIdleCallback(function() {
            // Add any deferred initializations here
            document.body.classList.add('animations-ready');
        });
    } else {
        setTimeout(function() {
            document.body.classList.add('animations-ready');
        }, 100);
    }

    // ============================================
    // Handle Reduced Motion Preference
    // ============================================
    const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');

    function handleReducedMotion() {
        if (prefersReducedMotion.matches) {
            // Disable auto-advancing carousel
            clearInterval(carouselInterval);

            // Add class for CSS to handle
            document.body.classList.add('reduced-motion');
        }
    }

    handleReducedMotion();
    prefersReducedMotion.addEventListener('change', handleReducedMotion);

    // ============================================
    // Initialize
    // ============================================
    function init() {
        updateNavbar();
        updateCarousel();
        highlightNavLink();
    }

    // Run on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

})();
