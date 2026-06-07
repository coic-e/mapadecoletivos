import { useState, useRef, MutableRefObject } from 'react';
import { useThree, useFrame, Viewport, Camera } from '@react-three/fiber';
import * as THREE from 'three';
import { useGLTF } from '@react-three/drei';
import { UseThreeElementReturn } from '../components/ThreeElement';

interface Data {
  x: number;
  y: number;
  z: number;
  rX: number;
  rY: number;
  rZ: number;
}

export function useThreeElement(z: number, gltfPath: string): UseThreeElementReturn  {
  const ref = useRef() as React.RefObject<THREE.Mesh>;
  const [clicked, setClicked] = useState(false);
  const { viewport, camera } = useThree();

  const { nodes, materials } = useGLTF(gltfPath);
  const { width, height } = viewport.getCurrentViewport(camera as Camera, [0, 0, z]);

  const [data] = useState<Data>({
    x: THREE.MathUtils.randFloatSpread(2),
    y: THREE.MathUtils.randFloatSpread(height),
    z: z,
    rX: Math.random() * Math.PI,
    rY: Math.random() * Math.PI,
    rZ: Math.random() * Math.PI,
  });

  useFrame((state) => {
    if (!ref.current) return;
    ref.current.rotation.set(
      (data.rX += clicked ? 0.1 : 0.001),
      (data.rY += clicked ? 0.1 : 0.001),
      (data.rZ += clicked ? 0.1 : 0.001)
    );
    if (clicked) data.z -= 1;
    if (data.z < -200) {
      data.z = z;
      data.y = -height;
      setClicked(false);
    }
    ref.current.position.set(data.x * width, (data.y += 0.025), data.z);
    if (data.y > height) {
      data.y = -height;
    }
  });

  return { ref, nodes, materials, clicked, setClicked };
}