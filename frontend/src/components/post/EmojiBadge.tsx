import { Box, Text } from "@chakra-ui/react";

interface EmojiBadgeProps {
  emoji: string;
  count: number;
  onClick?: () => void;
}

function EmojiBadge({ emoji, count, onClick }: EmojiBadgeProps) {
  return (
    <Box
      display="flex"
      alignItems="center"
      bg="gray.100"
      borderRadius="md"
      p={2}
      minW="60px"
      justifyContent="space-between"
      border="black solid 1px"
      marginRight="1em"
      onClick={() => {
        if (onClick) onClick();
      }}
    >
      <Text fontSize="xl" mr={2}>
        {emoji}
      </Text>
      <Text fontSize="md" fontWeight="bold">
        {count}
      </Text>
    </Box>
  );
}

export default EmojiBadge;
