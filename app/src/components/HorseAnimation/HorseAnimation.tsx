import React, { useEffect, useRef } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import "./horse-animation.css";

const HorseAnimation: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    const container = containerRef.current;
    let camera: THREE.PerspectiveCamera;
    let scene: THREE.Scene;
    let renderer: THREE.WebGLRenderer;
    let mesh: THREE.Mesh;
    let mixer: THREE.AnimationMixer;
    let animationId: number;

    const init = async () => {
      // Get viewport dimensions for responsive scaling
      const isMobile = window.innerWidth < 768;
      const isTablet = window.innerWidth >= 768 && window.innerWidth < 1024;
      
      // Adjust camera and scale based on screen size
      const cameraZ = isMobile ? -8 : isTablet ? -7 : -6;
      const horseScale = isMobile ? 0.015 : isTablet ? 0.018 : 0.02;
      
      // Camera setup
      camera = new THREE.PerspectiveCamera(
        45,
        container.clientWidth / container.clientHeight,
        1,
        1000
      );
      camera.position.set(2, 3, cameraZ);
      camera.lookAt(0, 1, 0);

      // Scene setup
      scene = new THREE.Scene();
      scene.background = null;
      scene.fog = new THREE.Fog(0x000000, 10, 50);

      // Lighting
      const hemiLight = new THREE.HemisphereLight(0xffffff, 0x8d8d8d, 3);
      hemiLight.position.set(0, 20, 0);
      scene.add(hemiLight);

      const dirLight = new THREE.DirectionalLight(0xffffff, 3);
      dirLight.position.set(3, 10, 10);
      dirLight.castShadow = true;
      dirLight.shadow.camera.top = 2;
      dirLight.shadow.camera.bottom = -2;
      dirLight.shadow.camera.left = -2;
      dirLight.shadow.camera.right = 2;
      dirLight.shadow.camera.near = 0.1;
      dirLight.shadow.camera.far = 40;
      scene.add(dirLight);

      // Load horse model
      const loader = new GLTFLoader();
      try {
        const gltf = await loader.loadAsync("https://threejs.org/examples/models/gltf/Horse.glb");

        mesh = gltf.scene.children[0] as THREE.Mesh;
        mesh.scale.set(horseScale, horseScale, horseScale);
        mesh.position.y = 0;
        scene.add(mesh);

        // Setup animation
        mixer = new THREE.AnimationMixer(mesh);
        mixer.clipAction(gltf.animations[0]).setDuration(1).play();
      } catch (error) {
        console.error("Error loading horse model:", error);
      }

      // Renderer setup
      renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
      renderer.setPixelRatio(window.devicePixelRatio);
      renderer.setSize(container.clientWidth, container.clientHeight);
      renderer.shadowMap.enabled = true;
      container.appendChild(renderer.domElement);

      // Handle window resize
      const handleResize = () => {
        if (!container) return;
        
        const newIsMobile = window.innerWidth < 768;
        const newIsTablet = window.innerWidth >= 768 && window.innerWidth < 1024;
        const newCameraZ = newIsMobile ? -8 : newIsTablet ? -7 : -6;
        const newHorseScale = newIsMobile ? 0.015 : newIsTablet ? 0.018 : 0.02;
        
        // Update camera
        camera.aspect = container.clientWidth / container.clientHeight;
        camera.position.z = newCameraZ;
        camera.updateProjectionMatrix();
        
        // Update horse scale
        if (mesh) {
          mesh.scale.set(newHorseScale, newHorseScale, newHorseScale);
        }
        
        renderer.setSize(container.clientWidth, container.clientHeight);
      };
      window.addEventListener("resize", handleResize);

      // Animation loop
      const clock = new THREE.Clock();
      const animate = () => {
        animationId = requestAnimationFrame(animate);

        const delta = clock.getDelta();
        if (mixer) mixer.update(delta);

        // Rotate the horse slowly
        if (mesh) {
          mesh.rotation.y += 0.005;
        }

        renderer.render(scene, camera);
      };

      animate();

      // Cleanup
      return () => {
        window.removeEventListener("resize", handleResize);
        if (animationId) cancelAnimationFrame(animationId);
        if (renderer) {
          container.removeChild(renderer.domElement);
          renderer.dispose();
        }
      };
    };

    const cleanup = init();

    return () => {
      cleanup.then((cleanupFn) => cleanupFn && cleanupFn());
    };
  }, []);

  return <div ref={containerRef} className="horse-animation-container" />;
};

export default HorseAnimation;
