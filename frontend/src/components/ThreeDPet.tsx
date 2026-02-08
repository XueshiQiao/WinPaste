import { useRef, useEffect } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { useGLTF, useAnimations, PerspectiveCamera, Center } from '@react-three/drei';
import * as THREE from 'three';

// Restore the simpler, working model structure
function FoxModel() {
  const group = useRef<THREE.Group>(null);
  const { scene, animations } = useGLTF('/Fox.gltf');
  const { actions } = useAnimations(animations, group);

  useEffect(() => {
    // Log for debugging
    console.log('FoxModel Mounted');
    console.log('Animations found:', Object.keys(actions));

    // Simple animation logic: Find 'Walk', or use first available
    const walkAction = Object.keys(actions).find(key => key.toLowerCase().includes('walk'));
    const actionName = walkAction || Object.keys(actions)[0];
    
    const action = actions[actionName];
    if (action) {
      console.log('Playing action:', actionName);
      action.reset().fadeIn(0.5).play();
    }
  }, [actions]);

  useFrame((state) => {
    if (!group.current) return;
    
    // Simple mouse tracking
    const { x, y } = state.mouse;
    const targetRotationY = x * 0.4;
    const targetRotationX = -y * 0.2;

    group.current.rotation.y = THREE.MathUtils.lerp(group.current.rotation.y, targetRotationY, 0.1);
    group.current.rotation.x = THREE.MathUtils.lerp(group.current.rotation.x, targetRotationX, 0.1);
  });

  return (
    <group ref={group} dispose={null}>
       {/* 
          Reverting to specific vector scale [0.7, 0.7, 0.7] 
          and keeping Center as it was in the working version.
       */}
       <Center>
          <primitive object={scene} scale={[0.7, 0.7, 0.7]} />
       </Center>
    </group>
  );
}

export function ThreeDPet({ theme = 'light' }: { theme?: 'light' | 'dark' }) {
  return (
    <div className="h-full w-full">
      <Canvas shadows>
        {/* Revert camera to original working position [0, 0, 5] */}
        <PerspectiveCamera makeDefault position={[0, 0, 5]} fov={35} />
        
        <ambientLight intensity={theme === 'dark' ? 0.4 : 0.8} />
        <spotLight position={[10, 10, 10]} angle={0.15} penumbra={1} intensity={1} castShadow />
        <pointLight position={[-10, -10, -10]} intensity={0.5} />
        
        {/* Removed Environment and Suspense to match original working state */}
        <FoxModel />
      </Canvas>
    </div>
  );
}
