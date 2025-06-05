// src/components/AsciinemaPlayerManual.js
import React, { useEffect, useRef } from "react";

const AsciinemaPlayerManual = ({ src, ...options }) => {
  const playerRef = useRef(null); // To hold the div element

  useEffect(() => {
    let player; // To store the player instance for cleanup

    // Ensure the AsciinemaPlayer global object is available
    if (typeof window.AsciinemaPlayer !== "undefined" && playerRef.current) {
      try {
        player = window.AsciinemaPlayer.create(
          src,
          playerRef.current, // Pass the actual DOM element
          options, // Pass all other props as options
        );
        console.log(
          "Asciinema player created via React component for src:",
          src,
        );
      } catch (e) {
        console.error("Error creating Asciinema player in React component:", e);
      }
    } else if (!playerRef.current) {
      console.warn("AsciinemaPlayer: playerRef.current is not available.");
    } else {
      console.warn(
        "AsciinemaPlayer global object not found. Make sure the player script is loaded.",
      );
    }

    // Cleanup function when the component unmounts
    return () => {
      if (player && typeof player.dispose === "function") {
        // Some players have a dispose method, check asciinema docs if this is applicable
        // For asciinema-player, often just removing the container's content or the container itself is enough
        // or if it provides a specific dispose/destroy method.
        // If not, manually clear the container:
        // if (playerRef.current) {
        //   playerRef.current.innerHTML = '';
        // }
        console.log(
          "Asciinema player instance cleanup (if dispose method exists or manual clear needed)",
        );
      }
      // If the create method returns a player instance with a dispose() method:
      // if (player && player.dispose) {
      //   player.dispose();
      // }
    };
  }, [src, options]); // Re-run effect if src or options change

  // Apply a key to the div if src changes, to force re-creation if needed,
  // though the useEffect dependency array should handle changes.
  return <div ref={playerRef} key={src}></div>;
};

export default AsciinemaPlayerManual;
