import { Box, Text } from "@chakra-ui/react";

interface EmojiBadgeProps {
  emoji: string;
  count: number;
}

function EmojiBadge({ emoji, count }: EmojiBadgeProps) {
  return (
    <Box
      display="flex"
      alignItems="center"
      bg="gray.100"
      borderRadius="md"
      p={2}
      minW="60px"
      justifyContent="space-between"
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
