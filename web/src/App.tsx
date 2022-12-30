import {
  Box,
  Heading,
  Container,
  Text,
  Stack,
  Link,
} from '@chakra-ui/react';

import CreateLink from '../components/new_link'

function App() {

  return (
    <>
      <Container maxW={'3xl'} paddingTop='100'>
        <Stack
          as={Box}
          textAlign={'center'}
          spacing={{ base: 8, md: 14 }}
          py={{ base: 20, md: 36 }}>
          <Heading
            fontWeight={600}
            fontSize={{ base: '2xl', sm: '4xl', md: '6xl' }}
            lineHeight={'110%'}>
            No more big long links, <br />
            <Text as={'span'} color={'teal.100'}>
              only short sweet once
            </Text>
          </Heading>
          <Text color={'gray.500'}>
            A very simple url shortner made with Rust and React
          </Text>
          <CreateLink />
        </Stack>
      </Container>
      <Container centerContent paddingTop='200' >
        <Text lineHeight='tall'>
          Made with ❤️ by <Link href='https://github.com/snxk' isExternal rounded='full' bg='teal.100' color='black' px='2' py='1'> Sayan Mallick </Link>
        </Text>
      </Container>
    </>
  )
}

export default App
