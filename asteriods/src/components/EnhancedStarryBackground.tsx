import React, { useRef, useEffect } from 'react';
import * as THREE from 'three';

interface ShootingStarData {
	lifeTime: number;
	maxLife: number;
}

interface MousePosition {
	x: number;
	y: number;
	isDragging: boolean;
}

const EnhancedStarryBackground: React.FC = () => {
	const mountRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		const currentRef = mountRef.current;
		if (!currentRef) return;

		// Scene setup
		const scene = new THREE.Scene();

		// Camera setup
		const camera = new THREE.PerspectiveCamera(
			75,
			window.innerWidth / window.innerHeight,
			0.1,
			1000
		);
		camera.position.z = 5;

		// Mouse control setup
		const mouse: MousePosition = {
			x: 0,
			y: 0,
			isDragging: false
		};

		// Renderer setup
		const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
		renderer.setSize(window.innerWidth, window.innerHeight);
		renderer.setClearColor(0x000000); // Pure black background
		currentRef.appendChild(renderer.domElement);

		// Create stars with different sizes
		const createStars = (): THREE.Group => {
			const stars = new THREE.Group();

			// Small stars (background)
			const smallStarGeometry = new THREE.BufferGeometry();
			const smallStarMaterial = new THREE.PointsMaterial({
				color: 0xffffff,
				size: 0.5,
				transparent: true,
			});

			const smallStarVertices: number[] = [];
			const smallStarOpacities: number[] = [];

			for (let i = 0; i < 15000; i++) { // Increased from 5000 to 15000
				const x = (Math.random() - 0.5) * 2000;
				const y = (Math.random() - 0.5) * 2000;
				const z = (Math.random() - 0.5) * 2000;
				smallStarVertices.push(x, y, z);
				smallStarOpacities.push(Math.random());
			}

			smallStarGeometry.setAttribute(
				'position',
				new THREE.Float32BufferAttribute(smallStarVertices, 3)
			);

			const smallStars = new THREE.Points(smallStarGeometry, smallStarMaterial);
			(smallStars.userData as { opacities: number[] }).opacities = smallStarOpacities;
			stars.add(smallStars);

			// Medium stars
			const mediumStarGeometry = new THREE.BufferGeometry();
			const mediumStarMaterial = new THREE.PointsMaterial({
				color: 0xeeeeff,
				size: 0.1,
				transparent: true,
			});

			const mediumStarVertices: number[] = [];
			const mediumStarOpacities: number[] = [];

			for (let i = 0; i < 7500; i++) { // Increased from 2500 to 7500
				const x = (Math.random() - 0.5) * 1500;
				const y = (Math.random() - 0.5) * 1500;
				const z = (Math.random() - 0.5) * 1500;
				mediumStarVertices.push(x, y, z);
				mediumStarOpacities.push(Math.random());
			}

			mediumStarGeometry.setAttribute(
				'position',
				new THREE.Float32BufferAttribute(mediumStarVertices, 3)
			);

			const mediumStars = new THREE.Points(mediumStarGeometry, mediumStarMaterial);
			(mediumStars.userData as { opacities: number[] }).opacities = mediumStarOpacities;
			stars.add(mediumStars);

			// Large twinkling stars
			const largeStarGeometry = new THREE.BufferGeometry();
			const largeStarMaterial = new THREE.PointsMaterial({
				color: 0xffffff,
				size: 0.15,
				transparent: true,
			});

			const largeStarVertices: number[] = [];
			const opacities: number[] = [];
			for (let i = 0; i < 3000; i++) { // Increased from 1000 to 3000
				const x = (Math.random() - 0.5) * 1000;
				const y = (Math.random() - 0.5) * 1000;
				const z = (Math.random() - 0.5) * 1000;
				largeStarVertices.push(x, y, z);

				// Store initial opacity for twinkling effect
				opacities.push(Math.random());
			}

			largeStarGeometry.setAttribute(
				'position',
				new THREE.Float32BufferAttribute(largeStarVertices, 3)
			);

			const largeStars = new THREE.Points(largeStarGeometry, largeStarMaterial);
			// Type assertion to add custom user data
			(largeStars.userData as { opacities: number[] }).opacities = opacities;
			stars.add(largeStars);

			return stars;
		};

		// Create shooting stars
		const createShootingStar = (): THREE.Line => {
			const material = new THREE.LineBasicMaterial({
				color: 0xffffff,
				transparent: true,
				opacity: 1
			});

			// Random start and end points
			const x1 = (Math.random() - 0.5) * 800;
			const y1 = Math.random() * 400;
			const x2 = x1 - Math.random() * 200 - 100;
			const y2 = y1 - Math.random() * 200 - 100;

			const points = [];
			points.push(new THREE.Vector3(x1, y1, -500));
			points.push(new THREE.Vector3(x2, y2, -500));

			const geometry = new THREE.BufferGeometry().setFromPoints(points);
			const line = new THREE.Line(geometry, material);

			// Store animation data with TypeScript interface
			line.userData = {
				lifeTime: 0,
				maxLife: Math.random() * 1.5 + 0.5, // Random lifetime between 0.5 and 2 seconds
			} as ShootingStarData;

			return line;
		};

		// Add stars to the scene
		const stars = createStars();
		scene.add(stars);

		// Array to store shooting stars
		const shootingStars: THREE.Line[] = [];

		// Handle window resize
		const handleResize = (): void => {
			const width = window.innerWidth;
			const height = window.innerHeight;

			camera.aspect = width / height;
			camera.updateProjectionMatrix();

			renderer.setSize(width, height);
		};

		window.addEventListener('resize', handleResize);

		// Animation variables
		let lastTime = 0;
		let shootingStarTimer = 0;

		// Animation loop
		const animate = (time: number): void => {
			const delta = (time - lastTime) / 1000; // Convert to seconds
			lastTime = time;

			requestAnimationFrame(animate);

			// Rotate stars slightly for subtle movement (only if not being dragged)
			if (!mouse.isDragging) {
				stars.rotation.x += 0.0001;
				stars.rotation.y += 0.0002;
			}

			// Handle twinkling effect for small stars
			const smallStars = stars.children[0] as THREE.Points;
			const smallOpacities = (smallStars.userData as { opacities: number[] }).opacities;

			if (smallStars.material instanceof THREE.PointsMaterial) {
				smallStars.material.opacity = Math.sin(time * 0.0005) * 0.3 + 0.7;
			}

			// Handle twinkling effect for medium stars
			const mediumStars = stars.children[1] as THREE.Points;
			const mediumOpacities = (mediumStars.userData as { opacities: number[] }).opacities;

			if (mediumStars.material instanceof THREE.PointsMaterial) {
				mediumStars.material.opacity = Math.sin(time * 0.0007) * 0.4 + 0.6;
			}

			// Handle twinkling effect for large stars
			const largeStars = stars.children[2] as THREE.Points;
			const opacities = (largeStars.userData as { opacities: number[] }).opacities;

			for (let i = 0; i < opacities.length; i++) {
				// Oscillate opacity with a sine wave and some randomness
				opacities[i] += delta * (Math.random() * 0.5);
				if (largeStars.material instanceof THREE.PointsMaterial) {
					// More pronounced twinkling for large stars
					largeStars.material.opacity = Math.sin(time * 0.001) * 0.5 + 0.5;
				}
			}

			// Manage shooting stars
			shootingStarTimer -= delta;
			if (shootingStarTimer <= 0) {
				// Create a new shooting star
				const shootingStar = createShootingStar();
				scene.add(shootingStar);
				shootingStars.push(shootingStar);

				// Set timer for next shooting star (between 1 and 6 seconds)
				shootingStarTimer = Math.random() * 5 + 1;
			}

			// Update and remove shooting stars
			for (let i = shootingStars.length - 1; i >= 0; i--) {
				const star = shootingStars[i];
				const userData = star.userData as ShootingStarData;
				userData.lifeTime += delta;

				// Fade out shooting star
				if (star.material instanceof THREE.LineBasicMaterial) {
					star.material.opacity = 1 - (userData.lifeTime / userData.maxLife);
				}

				// Remove if lifetime exceeded
				if (userData.lifeTime >= userData.maxLife) {
					scene.remove(star);
					shootingStars.splice(i, 1);
				}
			}

			renderer.render(scene, camera);
		};

		animate(0);

		// Clean up
		return () => {
			window.removeEventListener('resize', handleResize);

			// Remove mouse event listeners
			currentRef.removeEventListener('mousedown', handleMouseDown);
			window.removeEventListener('mousemove', handleMouseMove);
			window.removeEventListener('mouseup', handleMouseUp);
			currentRef.removeEventListener('mouseleave', handleMouseLeave);

			// Remove all shooting stars
			shootingStars.forEach(star => scene.remove(star));

			// Dispose of geometries and materials
			stars.children.forEach(starGroup => {
				if (starGroup instanceof THREE.Points) {
					starGroup.geometry.dispose();
					if (starGroup.material instanceof THREE.Material) {
						starGroup.material.dispose();
					} else if (Array.isArray(starGroup.material)) {
						starGroup.material.forEach(material => material.dispose());
					}
				}
			});

			currentRef.removeChild(renderer.domElement);
		};
	}, []);

	return <div ref={mountRef} style={{ width: '100%', height: '100%', position: 'fixed', top: 0, left: 0, zIndex: -1 }} />;
};

export default EnhancedStarryBackground;
