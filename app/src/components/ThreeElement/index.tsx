import { useThreeElement } from "../../hooks/useThreeElement";
import * as THREE from "three";

export interface UseThreeElementReturn {
  ref: React.RefObject<THREE.Mesh>;
  nodes: { [key: string]: THREE.Object3D };
  materials: { [key: string]: THREE.Material };
  clicked: boolean;
  setClicked: React.Dispatch<React.SetStateAction<boolean>>;
}

export default function ThreeElement({ z }: any) {
  const gltfPath = "/ferradura-v1-v2-transformed.glb";
  const { ref, nodes, materials, clicked, setClicked } = useThreeElement(
    z,
    gltfPath
  );

  return (
    <mesh
      ref={ref}
      scale={0.1}
      geometry={nodes.Horse_Shoe_lambert2_0.geometry}
      material={materials.lambert2}
      material-emissive="#F103E5"
      onPointerDown={() => setClicked(!clicked)}
    />
  );
}
