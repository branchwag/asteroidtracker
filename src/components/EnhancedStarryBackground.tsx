'use client';

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

		const scene = new THREE.Scene();

		const camera = new THREE.PerspectiveCamera(
			75,
			window.innerWidth / window.innerHeight,
			0.1,
			1000
		);
		camera.position.z = 5;

		const mouse: MousePosition = {
			x: 0,
			y: 0,
			isDragging: false
		};

		const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
		renderer.setSize(window.innerWidth, window.innerHeight);
		renderer.setClearColor(0x000000);
		currentRef.appendChild(renderer.domElement);

		const createCircleTexture = (): THREE.Texture => {
			const size = 64;
			const canvas = document.createElement('canvas');
			canvas.width = size;
			canvas.height = size;
			const ctx = canvas.getContext('2d');
			if (!ctx) throw new Error('Canvas 2D context not supported.');
			ctx.beginPath();
			ctx.arc(size / 2, size / 2, size / 2, 0, Math.PI * 2);
			ctx.fillStyle = 'white';
			ctx.fill();
			const texture = new THREE.CanvasTexture(canvas);
			texture.minFilter = THREE.LinearFilter;
			texture.magFilter = THREE.LinearFilter;
			texture.format = THREE.RGBAFormat;
			return texture;
		};

		const createStarLayer = (
			count: number,
			color: number,
			size: number,
			spread: number,
			speedMin: number,
			speedMax: number,
			amplitude: number
		): THREE.Points => {
			const geometry = new THREE.BufferGeometry();
			const starTexture = createCircleTexture();
			const material = new THREE.ShaderMaterial({
				uniforms: {
					uTime: { value: 0 },
					uTexture: { value: starTexture },
				},
				vertexShader: `
					attribute float aOpacity;
					attribute float aTwinkleSpeed;
					attribute float aTwinkleOffset;

					varying float vOpacity;
					uniform float uTime;
					
					void main() {
						float twinkle = sin(uTime * aTwinkleSpeed + aTwinkleOffset);
						vOpacity = aOpacity + twinkle * 0.5;

						gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
						gl_PointSize = 1.5;
					}
				`,
				fragmentShader: `
					uniform sampler2D uTexture;
					varying float vOpacity;

					void main() {
						vec4 texColor = texture2D(uTexture, gl_PointCoord);
						gl_FragColor = vec4(texColor.rgb, texColor.a * vOpacity);
					}
				`,
				transparent: true,
				depthWrite: false
			});

			const vertices: number[] = [];
			const aOpacity: number[] = [];
			const aTwinkleSpeed: number[] = [];
			const aTwinkleOffset: number[] = [];

			for (let i = 0; i < count; i++) {
				vertices.push(
					(Math.random() - 0.5) * spread,
					(Math.random() - 0.5) * spread,
					(Math.random() - 0.5) * spread
				);
				aOpacity.push(Math.random() * 0.5 + 0.5);
				aTwinkleSpeed.push(Math.random() * (speedMax - speedMin) + speedMin);
				aTwinkleOffset.push(Math.random() * Math.PI * 2);
			}

			geometry.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
			geometry.setAttribute('aOpacity', new THREE.Float32BufferAttribute(aOpacity, 1));
			geometry.setAttribute('aTwinkleSpeed', new THREE.Float32BufferAttribute(aTwinkleSpeed, 1));
			geometry.setAttribute('aTwinkleOffset', new THREE.Float32BufferAttribute(aTwinkleOffset, 1));

			const stars = new THREE.Points(geometry, material);
			stars.userData = {
				aOpacity,
				aTwinkleSpeed,
				aTwinkleOffset,
				amplitude,
			};

			return stars;
		};

		const createStars = (): THREE.Group => {
			const group = new THREE.Group();

			const smallStars = createStarLayer(
				1500,
				0xffffff,
				0.6,
				2000,
				0.001,
				0.005,
				0.05
			);

			const mediumStars = createStarLayer(
				750,
				0xeeeeff,
				0.8,
				1500,
				0.01,
				0.05,
				0.03
			);

			const largeStars = createStarLayer(
				3000,
				0xffffff,
				0.15,
				1000,
				0.005,
				0.01,
				0.2
			);

			group.add(smallStars, mediumStars, largeStars);
			return group;
		};

		const createShootingStar = (): THREE.Line => {
			const material = new THREE.LineBasicMaterial({
				color: 0xffffff,
				transparent: true,
				opacity: 1
			});

			const x1 = (Math.random() - 0.5) * 800;
			const y1 = Math.random() * 400;
			const x2 = x1 - Math.random() * 200 - 100;
			const y2 = y1 - Math.random() * 200 - 100;

			const points = [new THREE.Vector3(x1, y1, -500), new THREE.Vector3(x2, y2, -500)];
			const geometry = new THREE.BufferGeometry().setFromPoints(points);
			const line = new THREE.Line(geometry, material);

			line.userData = {
				lifeTime: 0,
				maxLife: Math.random() * 1.5 + 0.5
			} as ShootingStarData;

			return line;
		};

		const stars = createStars();
		scene.add(stars);

		const shootingStars: THREE.Line[] = [];

		const handleResize = (): void => {
			const width = window.innerWidth;
			const height = window.innerHeight;
			camera.aspect = width / height;
			camera.updateProjectionMatrix();
			renderer.setSize(width, height);
		};

		window.addEventListener('resize', handleResize);

		let lastTime = 0;
		let shootingStarTimer = 0;

		const animate = (time: number): void => {
			const delta = (time - lastTime) / 1000;
			lastTime = time;

			requestAnimationFrame(animate);

			if (!mouse.isDragging) {
				stars.rotation.x += 0.0001;
				stars.rotation.y += 0.0002;
			}

			for (const starLayer of stars.children) {
				if (starLayer instanceof THREE.Points && starLayer.material instanceof THREE.ShaderMaterial) {
					starLayer.material.uniforms.uTime.value = time * 0.001;	//ms to s
				}
			}

			shootingStarTimer -= delta;
			if (shootingStarTimer <= 0) {
				const shootingStar = createShootingStar();
				scene.add(shootingStar);
				shootingStars.push(shootingStar);
				shootingStarTimer = Math.random() * 5 + 1;
			}

			for (let i = shootingStars.length - 1; i >= 0; i--) {
				const star = shootingStars[i];
				const userData = star.userData as ShootingStarData;
				userData.lifeTime += delta;

				if (star.material instanceof THREE.LineBasicMaterial) {
					star.material.opacity = 1 - userData.lifeTime / userData.maxLife;
				}

				if (userData.lifeTime >= userData.maxLife) {
					scene.remove(star);
					shootingStars.splice(i, 1);
				}
			}

			renderer.render(scene, camera);
		};

		animate(0);

		return () => {
			window.removeEventListener('resize', handleResize);
			shootingStars.forEach((star) => scene.remove(star));

			stars.children.forEach((starGroup) => {
				if (starGroup instanceof THREE.Points) {
					starGroup.geometry.dispose();
					if (starGroup.material instanceof THREE.Material) {
						starGroup.material.dispose();
					}
				}
			});

			currentRef.removeChild(renderer.domElement);
		};
	}, []);

	return (
		<div
			ref={mountRef}
			style={{
				width: '100%',
				height: '100%',
				position: 'fixed',
				top: 0,
				left: 0,
				zIndex: -1
			}}
		/>
	);
};

export default EnhancedStarryBackground;
